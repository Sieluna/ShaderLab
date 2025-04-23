use axum::http::StatusCode;
use thiserror::Error;

use super::ErrorResponse;

#[derive(Debug, Error)]
pub enum NotebookError {
    #[error("Notebook not found")]
    NotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("No changes provided")]
    NoChanges,
}

impl ErrorResponse for NotebookError {
    fn status_code(&self) -> StatusCode {
        match self {
            NotebookError::NotFound => StatusCode::NOT_FOUND,
            NotebookError::PermissionDenied => StatusCode::FORBIDDEN,
            NotebookError::NoChanges => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> String {
        self.to_string()
    }
}
