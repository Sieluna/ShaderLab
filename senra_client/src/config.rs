use senra_api::*;
use url::Url;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub http_url: Url,
    pub ws_url: Option<Url>,
    pub token: Option<String>,
    pub protocol_config: ProtocolConfig,
    pub timeout_ms: u64,
}

impl ClientConfig {
    pub fn new(http_url: Url) -> Self {
        Self {
            http_url,
            ws_url: None,
            token: None,
            protocol_config: ProtocolConfig::default(),
            timeout_ms: 30000,
        }
    }

    pub fn with_ws_url(mut self, ws_url: Url) -> Self {
        self.ws_url = Some(ws_url);
        self
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
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn json(http_url: Url) -> Self {
        Self::new(http_url).with_protocol(ProtocolConfig::json())
    }

    pub fn postcard(http_url: Url) -> Self {
        Self::new(http_url).with_protocol(ProtocolConfig::postcard())
    }
}
