use core::str::FromStr;

use alloc::format;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::error::ProtocolError;

pub trait Codec: Send + Sync + Clone {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError>;
    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError>;
    fn content_type(&self) -> &'static str;
}

#[derive(Debug, Clone, Default)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(data).map_err(ProtocolError::from)
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError> {
        serde_json::from_slice(data).map_err(ProtocolError::from)
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }
}

#[derive(Debug, Clone, Default)]
pub struct PostcardCodec;

impl Codec for PostcardCodec {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError> {
        postcard::to_allocvec(data).map_err(ProtocolError::from)
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError> {
        postcard::from_bytes(data).map_err(ProtocolError::from)
    }

    fn content_type(&self) -> &'static str {
        "application/octet-stream"
    }
}

#[derive(Debug, Clone)]
pub enum ProtocolCodec {
    Json(JsonCodec),
    Postcard(PostcardCodec),
}

impl Codec for ProtocolCodec {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError> {
        match self {
            Self::Json(codec) => codec.encode(data),
            Self::Postcard(codec) => codec.encode(data),
        }
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError> {
        match self {
            Self::Json(codec) => codec.decode(data),
            Self::Postcard(codec) => codec.decode(data),
        }
    }

    fn content_type(&self) -> &'static str {
        match self {
            Self::Json(codec) => codec.content_type(),
            Self::Postcard(codec) => codec.content_type(),
        }
    }
}

impl Default for ProtocolCodec {
    fn default() -> Self {
        Self::Json(JsonCodec)
    }
}

impl FromStr for ProtocolCodec {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json(JsonCodec)),
            "postcard" => Ok(Self::Postcard(PostcardCodec)),
            _ => Err(ProtocolError {
                message: format!("Invalid codec identifier: {}", s),
            }),
        }
    }
}

impl ProtocolCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json(_) => "json",
            Self::Postcard(_) => "postcard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        id: u32,
        message: String,
    }

    #[test]
    fn test_json_codec() {
        let codec = JsonCodec::default();
        let data = TestRequest {
            id: 42,
            message: "test".to_string(),
        };

        let encoded = codec.encode(&data).unwrap();
        let decoded: TestRequest = codec.decode(&encoded).unwrap();

        assert_eq!(data, decoded);
        assert_eq!(codec.content_type(), "application/json");
    }

    #[test]
    fn test_postcard_codec() {
        let codec = PostcardCodec::default();
        let data = TestRequest {
            id: 42,
            message: "test".to_string(),
        };

        let encoded = codec.encode(&data).unwrap();
        let decoded: TestRequest = codec.decode(&encoded).unwrap();

        assert_eq!(data, decoded);
        assert_eq!(codec.content_type(), "application/octet-stream");
    }
}
