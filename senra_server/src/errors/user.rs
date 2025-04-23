use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorResponse;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Invalid username")]
    InvalidUsername,

    #[error("Invalid email")]
    InvalidEmail,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserExists,

    #[error("No changes provided")]
    NoChanges,
}

impl ErrorResponse for UserError {
    fn status_code(&self) -> StatusCode {
        match self {
            UserError::InvalidUsername => StatusCode::BAD_REQUEST,
            UserError::InvalidEmail => StatusCode::BAD_REQUEST,
            UserError::InvalidPassword => StatusCode::BAD_REQUEST,
            UserError::UserNotFound => StatusCode::NOT_FOUND,
            UserError::UserExists => StatusCode::CONFLICT,
            UserError::NoChanges => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
