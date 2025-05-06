use alloc::string::{String, ToString};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ProtocolError {
    /// Codec operation failed (serialization/deserialization)
    #[error("Codec error: {message}")]
    Codec { message: String },

    /// Network transport error
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// Request timeout
    #[error("Request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Unknown or unclassified error
    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        Self::Codec {
            message: err.to_string(),
        }
    }
}

impl From<postcard::Error> for ProtocolError {
    fn from(err: postcard::Error) -> Self {
        Self::Codec {
            message: err.to_string(),
        }
    }
}
