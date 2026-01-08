use alloc::string::{String, ToString};

use thiserror::Error;

/// Codec operation error (serialization/deserialization)
#[derive(Debug, Clone, PartialEq, Error)]
#[error("Codec error: {message}")]
pub struct ProtocolError {
    pub message: String,
}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<postcard::Error> for ProtocolError {
    fn from(err: postcard::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}
