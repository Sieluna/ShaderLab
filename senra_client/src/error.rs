use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] crate::http::Error),

    #[error(transparent)]
    WebSocket(#[from] crate::ws::Error),

    #[error(transparent)]
    Client(#[from] crate::client::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error(transparent)]
    Protocol(#[from] senra_api::ProtocolError),
}

pub type Result<T> = core::result::Result<T, Error>;
