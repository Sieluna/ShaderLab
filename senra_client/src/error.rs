#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP-related errors
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    /// WebSocket-related errors  
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WebSocketError),

    /// Protocol-level errors
    #[error("Protocol error: {0}")]
    Protocol(#[from] senra_api::ProtocolError),

    /// Authentication and authorization errors
    #[error("Authentication error: {message}")]
    Auth { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Invalid input or request errors
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    /// Server responded with an error
    #[error("Server error: {status} - {message}")]
    Server { status: u16, message: String },

    /// Resource not found
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    /// Failed to build HTTP request
    #[error("Failed to build request: {source}")]
    RequestBuild {
        #[source]
        source: reqwest::Error,
    },

    /// Network request failed
    #[error("Network request failed: {source}")]
    Network {
        #[source]
        source: reqwest::Error,
    },

    /// Failed to read response
    #[error("Failed to read response: {source}")]
    Response {
        #[source]
        source: reqwest::Error,
    },

    /// Unsupported HTTP method
    #[error("Unsupported HTTP method: {method}")]
    UnsupportedMethod { method: String },

    /// Invalid URL format
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
}

#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    /// Connection failed
    #[error("WebSocket connection failed: {message}")]
    ConnectionFailed { message: String },

    /// Not connected
    #[error("WebSocket not connected")]
    NotConnected,

    /// Failed to send message
    #[error("Failed to send WebSocket message: {message}")]
    SendFailed { message: String },

    /// Failed to receive message
    #[error("Failed to receive WebSocket message: {message}")]
    ReceiveFailed { message: String },

    /// Connection closed unexpectedly  
    #[error("WebSocket connection closed unexpectedly")]
    ConnectionClosed,

    /// Invalid message format
    #[error("Invalid WebSocket message format: {message}")]
    InvalidMessage { message: String },
}

impl From<reqwest::Error> for HttpError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_builder() {
            HttpError::RequestBuild { source: err }
        } else if err.is_request() || err.is_timeout() {
            HttpError::Network { source: err }
        } else {
            HttpError::Response { source: err }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
