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

#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    pub codec: ProtocolCodec,
    pub timeout_ms: Option<u64>,
}

impl ProtocolConfig {
    pub fn new(codec: ProtocolCodec) -> Self {
        Self {
            codec,
            timeout_ms: Some(30000),
        }
    }

    pub fn json() -> Self {
        Self::new(ProtocolCodec::Json(JsonCodec))
    }

    pub fn postcard() -> Self {
        Self::new(ProtocolCodec::Postcard(PostcardCodec))
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self::json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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

    #[test]
    fn test_protocol_config() {
        let config = ProtocolConfig::json().with_timeout(60000);
        assert_eq!(config.timeout_ms, Some(60000));

        let config = ProtocolConfig::postcard();
        assert_eq!(config.timeout_ms, Some(30000));
    }
}
