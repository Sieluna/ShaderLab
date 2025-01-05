mod auth;
mod notebook;
mod shader;
mod user;

pub use auth::AuthError;
pub use notebook::NotebookError;
pub use shader::ShaderError;
pub use user::UserError;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use time::OffsetDateTime;

pub trait ErrorResponse: std::fmt::Display {
    fn status_code(&self) -> StatusCode;
    fn error_message(&self) -> String;
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),

    #[error("User error: {0}")]
    UserError(#[from] UserError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Notebook error: {0}")]
    NotebookError(#[from] NotebookError),

    #[error("Shader error: {0}")]
    ShaderError(#[from] ShaderError),
}

impl ErrorResponse for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::AuthError(e) => e.status_code(),
            AppError::UserError(e) => e.status_code(),
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotebookError(e) => e.status_code(),
            AppError::ShaderError(e) => e.status_code(),
        }
    }

    fn error_message(&self) -> String {
        match self {
            AppError::AuthError(e) => e.error_message(),
            AppError::UserError(e) => e.error_message(),
            AppError::DatabaseError(e) => e.to_string(),
            AppError::ValidationError(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::InternalError(msg) => msg.clone(),
            AppError::NotebookError(e) => e.error_message(),
            AppError::ShaderError(e) => e.error_message(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({
            "error": self.error_message(),
            "timestamp": OffsetDateTime::now_utc().to_string(),
        });

        (self.status_code(), Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
