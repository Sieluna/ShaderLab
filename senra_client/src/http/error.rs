use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unsupported HTTP method: {0}")]
    UnsupportedMethod(String),

    #[error("HTTP {status}: {message}")]
    Status { status: u16, message: String },

    #[error("Codec error: {0}")]
    Codec(#[from] senra_api::ProtocolError),

    #[error("HTTP error: {0}")]
    Transport(#[from] reqwest::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
