use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("WebSocket not connected")]
    NotConnected,

    #[error("WebSocket connection timeout")]
    Timeout,

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Codec error: {0}")]
    Codec(#[from] senra_api::ProtocolError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("WebSocket error: {0}")]
    Transport(#[from] tokio_tungstenite::tungstenite::Error),

    #[cfg(target_arch = "wasm32")]
    #[error("WebSocket error: {0}")]
    Transport(String),
}

pub type Result<T> = core::result::Result<T, Error>;
