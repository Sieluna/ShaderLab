use senra_api::*;

#[derive(Debug, Clone)]
pub enum Error {
    Protocol(ProtocolError),
    Network(String),
    Authentication(String),
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    Unknown(String),
}

impl From<ProtocolError> for Error {
    fn from(err: ProtocolError) -> Self {
        Self::Protocol(err)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Protocol(err) => write!(f, "Protocol error: {}", err),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Self::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
