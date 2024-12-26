use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Cursor;

use bcrypt::{DEFAULT_COST, hash};
use image::{ImageBuffer, ImageFormat, Rgba};
use sqlx::{QueryBuilder, SqlitePool};

use crate::errors::{AppError, Result, UserError};
use crate::models::{CreateUser, EditUser, User};

#[derive(Clone)]
pub struct UserService {
    pool: SqlitePool,
}

impl UserService {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn get_user(&self, user_id: i64) -> Result<User> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.ok_or(UserError::UserNotFound)?)
    }

    pub async fn create_user(&self, create_user: CreateUser) -> Result<User> {
        if create_user.username.is_empty() {
            Err(UserError::InvalidUsername)?;
        }

        if create_user.email.is_empty() {
            Err(UserError::InvalidEmail)?;
        }

        if create_user.password.is_empty() {
            Err(UserError::InvalidPassword)?;
        }

        let existing_user = sqlx::query("SELECT id FROM users WHERE username = $1 OR email = $2")
            .bind(&create_user.username)
            .bind(&create_user.email)
            .fetch_optional(&self.pool)
            .await?;

        if existing_user.is_some() {
            Err(UserError::UserExists)?;
        }

        let password_hash = hash(create_user.password, DEFAULT_COST)
            .map_err(|_| AppError::InternalError("Failed to hash password".to_string()))?;

        let mut hasher = DefaultHasher::new();
        create_user.username.hash(&mut hasher);
        let seed = hasher.finish();

        let mut bytes = Vec::new();
        let mut img = ImageBuffer::new(64, 64);

        let color = Rgba([
            ((seed >> 16) & 0xFF) as u8,
            ((seed >> 8) & 0xFF) as u8,
            (seed & 0xFF) as u8,
            255,
        ]);

        let grid_size = 5;
        let cell_size = 12;
        let padding = (64 - grid_size * cell_size) / 2;

        for y in 0..grid_size {
            for x in 0..(grid_size / 2 + 1) {
                let pattern = (seed >> (y * 3 + x)) & 0x7;
                if pattern % 2 == 0 {
                    for px in padding + x * cell_size..padding + (x + 1) * cell_size {
                        for py in padding + y * cell_size..padding + (y + 1) * cell_size {
                            *img.get_pixel_mut(px as u32, py as u32) = color;
                            if x < grid_size / 2 {
                                *img.get_pixel_mut((64 - px - 1) as u32, py as u32) = color;
                            }
                        }
                    }
                }
            }
        }

        img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)
            .ok();

        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (username, email, password, avatar)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, password, avatar, created_at, updated_at
            "#,
        )
        .bind(create_user.username)
        .bind(create_user.email)
        .bind(password_hash)
        .bind(bytes)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn edit_user(&self, user_id: i64, edit_user: EditUser) -> Result<User> {
        let mut query_builder = QueryBuilder::new("UPDATE users SET ");

        let mut has_changes = false;

        if let Some(username) = &edit_user.username {
            if username.is_empty() {
                Err(UserError::InvalidUsername)?;
            }
            query_builder.push("username = ").push_bind(username);
            has_changes = true;
        }

        if let Some(email) = &edit_user.email {
            if email.is_empty() {
                Err(UserError::InvalidEmail)?;
            }
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("email = ").push_bind(email);
            has_changes = true;
        }

        if let Some(password) = &edit_user.password {
            if password.is_empty() {
                Err(UserError::InvalidPassword)?;
            }
            let password_hash = hash(password, DEFAULT_COST)
                .map_err(|_| AppError::InternalError("Failed to hash password".to_string()))?;
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("password = ").push_bind(password_hash);
            has_changes = true;
        }

        if let Some(avatar) = &edit_user.avatar {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("avatar = ").push_bind(avatar);
            has_changes = true;
        }

        if !has_changes {
            Err(UserError::NoChanges)?;
        }

        query_builder
            .push(", updated_at = datetime('now') WHERE id = ")
            .push_bind(user_id);
        query_builder
            .push(" RETURNING id, username, email, password, avatar, created_at, updated_at");

        let user = query_builder
            .build_query_as::<User>()
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}
