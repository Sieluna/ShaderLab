use super::{ClientConfig, HttpClient, Result, WsClient};

use senra_api::*;

pub struct ClientBuilder {
    base_url: String,
    token: Option<String>,
    protocol_config: Option<ProtocolConfig>,
    timeout_ms: Option<u64>,
}

impl ClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
            protocol_config: None,
            timeout_ms: None,
        }
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn json_protocol(mut self) -> Self {
        self.protocol_config = Some(ProtocolConfig::json());
        self
    }

    pub fn postcard_protocol(mut self) -> Self {
        self.protocol_config = Some(ProtocolConfig::postcard());
        self
    }

    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn build_http(self) -> Result<HttpClient> {
        let mut config = ClientConfig::new(self.base_url);

        if let Some(token) = self.token {
            config = config.with_token(token);
        }

        if let Some(protocol) = self.protocol_config {
            config = config.with_protocol(protocol);
        }

        if let Some(timeout) = self.timeout_ms {
            config = config.with_timeout(timeout);
        }

        HttpClient::new(config)
    }

    pub async fn build_ws(self) -> Result<WsClient> {
        let mut config = ClientConfig::new(self.base_url);

        if let Some(token) = self.token {
            config = config.with_token(token);
        }

        if let Some(protocol) = self.protocol_config {
            config = config.with_protocol(protocol);
        }

        if let Some(timeout) = self.timeout_ms {
            config = config.with_timeout(timeout);
        }

        WsClient::new(config).await
    }

    #[cfg(target_arch = "wasm32")]
    pub fn build_api(self) -> Result<super::ApiClient> {
        let mut config = ClientConfig::new(self.base_url);

        if let Some(token) = self.token {
            config = config.with_token(token);
        }

        if let Some(protocol) = self.protocol_config {
            config = config.with_protocol(protocol);
        }

        if let Some(timeout) = self.timeout_ms {
            config = config.with_timeout(timeout);
        }

        super::ApiClient::new(&config.base_url)
            .map_err(|_| crate::error::Error::Unknown("Failed to create API client".to_string()))
    }
}

pub fn client(base_url: impl Into<String>) -> ClientBuilder {
    ClientBuilder::new(base_url)
}
