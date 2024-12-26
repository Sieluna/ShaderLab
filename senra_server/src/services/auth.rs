use std::sync::Arc;

use bcrypt::verify;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::errors::{AppError, AuthError, Result};
use crate::models::{LoginUser, User};

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

    pub async fn login(&self, login_user: LoginUser) -> Result<(User, String)> {
        if login_user.username.is_empty() {
            Err(AuthError::InvalidUsername)?;
        }

        if login_user.password.is_empty() {
            Err(AuthError::InvalidPassword)?;
        }

        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, username, email, password, avatar, created_at, updated_at
            FROM users
            WHERE username = $1
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

    pub async fn generate_token(&self, user_id: i64) -> Result<String> {
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
}
