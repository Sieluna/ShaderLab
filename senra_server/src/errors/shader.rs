use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorResponse;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Shader not found")]
    NotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Invalid shader data: {0}")]
    InvalidData(String),

    #[error("No changes provided")]
    NoChanges,
}

impl ErrorResponse for ShaderError {
    fn status_code(&self) -> StatusCode {
        match self {
            ShaderError::NotFound => StatusCode::NOT_FOUND,
            ShaderError::PermissionDenied => StatusCode::FORBIDDEN,
            ShaderError::CompilationError(_) => StatusCode::BAD_REQUEST,
            ShaderError::InvalidData(_) => StatusCode::BAD_REQUEST,
            ShaderError::NoChanges => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
