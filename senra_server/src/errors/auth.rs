use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorResponse;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid username")]
    InvalidUsername,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,
}

impl ErrorResponse for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::InvalidUsername => StatusCode::BAD_REQUEST,
            AuthError::InvalidPassword => StatusCode::BAD_REQUEST,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
