use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorResponse;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid username")]
    InvalidUsername,

    #[error("Invalid email")]
    InvalidEmail,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("User already exists")]
    UserExists,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("No changes provided")]
    NoChanges,
}

impl ErrorResponse for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::InvalidUsername => StatusCode::BAD_REQUEST,
            AuthError::InvalidEmail => StatusCode::BAD_REQUEST,
            AuthError::InvalidPassword => StatusCode::BAD_REQUEST,
            AuthError::UserExists => StatusCode::CONFLICT,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthError::NoChanges => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
