use senra_api::*;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub token: Option<String>,
    pub protocol_config: ProtocolConfig,
    pub timeout_ms: Option<u64>,
}

impl ClientConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
            protocol_config: ProtocolConfig::default(),
            timeout_ms: Some(30000),
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn with_protocol(mut self, protocol_config: ProtocolConfig) -> Self {
        self.protocol_config = protocol_config;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn json() -> Self {
        Self::new("").with_protocol(ProtocolConfig::json())
    }

    pub fn postcard() -> Self {
        Self::new("").with_protocol(ProtocolConfig::postcard())
    }
}
