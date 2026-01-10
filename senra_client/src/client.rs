use alloc::string::{String, ToString};

use reqwest::Client as HttpClient;
use thiserror::Error;
use url::Url;

use crate::config::ClientConfig;
use crate::ws::WsClient;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] crate::ws::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[cfg(target_arch = "wasm32")]
impl From<serde_wasm_bindgen::Error> for Error {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ApiClient {
    http: HttpClient,
    ws: WsClient,
    config: ClientConfig,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let timeout = config.timeout;
        #[cfg(not(target_arch = "wasm32"))]
        let http = HttpClient::builder().timeout(timeout).build()?;
        #[cfg(target_arch = "wasm32")]
        let http = HttpClient::builder().build()?;
        let ws = WsClient::new(timeout);

        Ok(Self {
            http,
            ws,
            config,
            token: None,
        })
    }

    pub fn http_url(&self) -> Url {
        self.config.http_url.clone()
    }

    pub fn ws_url(&self) -> Url {
        self.config.ws_url.clone()
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    pub fn ws(&self) -> &WsClient {
        &self.ws
    }

    pub fn ws_mut(&mut self) -> &mut WsClient {
        &mut self.ws
    }
}
