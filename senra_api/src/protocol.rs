use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use core::future::Future;
use core::pin::Pin;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// Serialization error
    SerializationError(String),
    /// Deserialization error  
    DeserializationError(String),
    /// Network transport error
    TransportError(String),
    /// Unknown error
    Unknown(String),
}

impl core::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProtocolError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ProtocolError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            ProtocolError::TransportError(msg) => write!(f, "Transport error: {}", msg),
            ProtocolError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {}

#[derive(Debug, Clone, Default)]
pub struct PipelineContext {
    /// Metadata for request routing and configuration
    pub metadata: BTreeMap<String, String>,
    /// Optional request ID for tracking
    pub request_id: Option<String>,
    /// Optional timestamp for timing
    pub timestamp: Option<u64>,
}

impl PipelineContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.metadata
            .insert("endpoint".to_string(), endpoint.into());
        self
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.metadata.insert("method".to_string(), method.into());
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, ProtocolError>> + Send>>;

pub trait Codec: Send + Sync + Clone {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError>;
    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError>;
    fn content_type(&self) -> &'static str;
}

pub trait Pipeline<Req, Res>: Send + Sync
where
    Req: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    Res: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    type Protocol;

    fn execute(&self, request: Req, context: PipelineContext) -> AsyncResult<Res>;
}

#[derive(Debug, Clone, Default)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(data).map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError> {
        serde_json::from_slice(data).map_err(|e| ProtocolError::DeserializationError(e.to_string()))
    }

    fn content_type(&self) -> &'static str {
        "application/json"
    }
}

#[derive(Debug, Clone, Default)]
pub struct PostcardCodec;

impl Codec for PostcardCodec {
    fn encode<T: Serialize>(&self, data: &T) -> Result<Vec<u8>, ProtocolError> {
        postcard::to_allocvec(data).map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    fn decode<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, ProtocolError> {
        postcard::from_bytes(data).map_err(|e| ProtocolError::DeserializationError(e.to_string()))
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
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestRequest {
        id: u32,
        message: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        success: bool,
        data: String,
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

    #[test]
    fn test_protocol_error_display() {
        let error = ProtocolError::SerializationError("test error".to_string());
        assert_eq!(error.to_string(), "Serialization error: test error");

        let error = ProtocolError::TransportError("network error".to_string());
        assert_eq!(error.to_string(), "Transport error: network error");
    }

    #[test]
    fn test_pipeline_context() {
        let context = PipelineContext::new()
            .with_endpoint("/api/test")
            .with_method("GET")
            .with_request_id("test-123")
            .with_metadata("key", "value");

        assert_eq!(
            context.metadata.get("endpoint"),
            Some(&"/api/test".to_string())
        );
        assert_eq!(context.metadata.get("method"), Some(&"GET".to_string()));
        assert_eq!(context.request_id, Some("test-123".to_string()));
        assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
    }
}
