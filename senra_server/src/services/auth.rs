use std::sync::Arc;

use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, SqlitePool};
use time::OffsetDateTime;

use crate::errors::{AppError, AuthError, Result};
use crate::models::{CreateUser, EditUser, LoginUser, User};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    exp: i64,
    iat: i64,
}

#[derive(Clone)]
pub struct AuthService {
    pool: SqlitePool,
    jwt_secret: Arc<str>,
}

impl AuthService {
    pub fn new(pool: &SqlitePool, jwt_secret: &str) -> Self {
        Self {
            pool: pool.clone(),
            jwt_secret: Arc::from(jwt_secret),
        }
    }

    pub async fn register(&self, create_user: CreateUser) -> Result<(User, String)> {
        if create_user.username.is_empty() {
            Err(AuthError::InvalidUsername)?;
        }

        if create_user.email.is_empty() {
            Err(AuthError::InvalidEmail)?;
        }

        if create_user.password.is_empty() {
            Err(AuthError::InvalidPassword)?;
        }

        let existing_user = sqlx::query("SELECT id FROM users WHERE username = ? OR email = ?")
            .bind(&create_user.username)
            .bind(&create_user.email)
            .fetch_optional(&self.pool)
            .await?;

        if existing_user.is_some() {
            Err(AuthError::UserExists)?;
        }

        let password_hash = hash(create_user.password, DEFAULT_COST)
            .map_err(|_| AppError::InternalError("Failed to hash password".to_string()))?;

        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (username, email, password)
            VALUES (?, ?, ?)
            RETURNING id, username, email, password, avatar_url, created_at, updated_at
            "#,
        )
        .bind(create_user.username)
        .bind(create_user.email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        let token = self.generate_token(user.id).await?;

        Ok((user, token))
    }

    pub async fn login(&self, login_user: LoginUser) -> Result<(User, String)> {
        if login_user.username.is_empty() {
            Err(AuthError::InvalidUsername)?;
        }

        if login_user.password.is_empty() {
            Err(AuthError::InvalidPassword)?;
        }

        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, username, email, password, avatar_url, created_at, updated_at
            FROM users
            WHERE username = ?
            "#,
        )
        .bind(login_user.username)
        .fetch_optional(&self.pool)
        .await?;

        let user = user.ok_or(AuthError::InvalidCredentials)?;
        let password_hash = user.password.clone().ok_or(AuthError::InvalidCredentials)?;

        if !verify(login_user.password, &password_hash)
            .map_err(|_| AppError::InternalError("Failed to verify password".to_string()))?
        {
            Err(AuthError::InvalidCredentials)?;
        }

        let token = self.generate_token(user.id).await?;

        Ok((user, token))
    }

    pub async fn authorize(&self, token: &str) -> Result<i64> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims.sub)
    }

    pub async fn refresh_token(&self, token: &str) -> Result<Option<String>> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let expires_in = token_data.claims.exp - now;

        if expires_in < 1800 {
            let claims = Claims {
                sub: token_data.claims.sub,
                exp: now + 3600 * 24,
                iat: now,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
            )
            .map_err(|_| AppError::InternalError("Failed to generate token".to_string()))?;

            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    async fn generate_token(&self, user_id: i64) -> Result<String> {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let claims = Claims {
            sub: user_id,
            exp: now + 3600 * 24,
            iat: now,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AppError::InternalError("Failed to generate token".to_string()))?;

        Ok(token)
    }

    pub async fn edit_user(&self, user_id: i64, edit_user: EditUser) -> Result<User> {
        let mut query_builder = QueryBuilder::new("UPDATE users SET ");

        let mut has_changes = false;

        if let Some(username) = &edit_user.username {
            if username.is_empty() {
                Err(AuthError::InvalidUsername)?;
            }
            query_builder.push("username = ").push_bind(username);
            has_changes = true;
        }

        if let Some(email) = &edit_user.email {
            if email.is_empty() {
                Err(AuthError::InvalidEmail)?;
            }
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("email = ").push_bind(email);
            has_changes = true;
        }

        if let Some(password) = &edit_user.password {
            if password.is_empty() {
                Err(AuthError::InvalidPassword)?;
            }
            let password_hash = hash(password, DEFAULT_COST)
                .map_err(|_| AppError::InternalError("Failed to hash password".to_string()))?;
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("password = ").push_bind(password_hash);
            has_changes = true;
        }

        if let Some(avatar_url) = &edit_user.avatar_url {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("avatar_url = ").push_bind(avatar_url);
            has_changes = true;
        }

        if !has_changes {
            Err(AuthError::NoChanges)?;
        }

        query_builder
            .push(", updated_at = datetime('now') WHERE id = ")
            .push_bind(user_id);
        query_builder
            .push(" RETURNING id, username, email, password, avatar_url, created_at, updated_at");

        let user = query_builder
            .build_query_as::<User>()
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}
