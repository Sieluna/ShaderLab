pub mod codec;
pub mod error;

pub use codec::{Codec, JsonCodec, PostcardCodec, ProtocolCodec, ProtocolConfig};
pub use error::ProtocolError;

use core::future::Future;
use core::pin::Pin;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

use serde::{Deserialize, Serialize};

pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, ProtocolError>> + Send>>;

pub trait Pipeline<Req, Res>: Send + Sync
where
    Req: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    Res: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    type Protocol;

    fn execute(&self, request: Req, context: PipelineContext) -> AsyncResult<Res>;
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestResponse {
        success: bool,
        data: String,
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
