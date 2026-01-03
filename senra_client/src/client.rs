use thiserror::Error;
use url::Url;

use crate::config::ClientConfig;
use crate::http::HttpClient;
use crate::ws::{Message, WsClient};

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] crate::http::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] crate::ws::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct ApiClient {
    http: HttpClient,
    ws: WsClient,
    config: ClientConfig,
}

impl ApiClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        Ok(Self {
            http: HttpClient::new(config.clone()).map_err(Error::Http)?,
            ws: WsClient::new(),
            config,
        })
    }

    pub async fn http_request<Req, Res>(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<Req>,
    ) -> Result<Res>
    where
        Req: serde::Serialize,
        Res: for<'de> serde::Deserialize<'de>,
    {
        self.http
            .request(method, endpoint, body)
            .await
            .map_err(Error::Http)
    }

    pub fn http_base_url(&self) -> &str {
        self.http.base_url()
    }

    pub fn http_token(&self) -> Option<&str> {
        self.http.token()
    }

    pub fn http_set_token(&mut self, token: String) {
        self.http.set_token(token);
    }

    pub fn http_clear_token(&mut self) {
        self.http.clear_token();
    }

    pub async fn ws_connect(&mut self, url: Url) -> Result<()> {
        self.ws.connect(url).await.map_err(Error::WebSocket)
    }

    pub async fn ws_disconnect(&mut self) -> Result<()> {
        self.ws.disconnect().await.map_err(Error::WebSocket)
    }

    pub async fn ws_send(&mut self, msg: Message) -> Result<()> {
        self.ws.send(msg).await.map_err(Error::WebSocket)
    }

    pub async fn ws_receive(&mut self) -> Result<Option<Message>> {
        self.ws.receive().await.map_err(Error::WebSocket)
    }

    pub fn ws_is_connected(&self) -> bool {
        self.ws.is_connected()
    }

    pub fn ws_url(&self) -> Option<&Url> {
        self.ws.url()
    }

    pub fn ws_token(&self) -> Option<&str> {
        self.config.token.as_deref()
    }

    pub fn set_config_token(&mut self, token: String) {
        self.config.token = Some(token.clone());
        self.http_set_token(token);
    }

    pub fn clear_config_token(&mut self) {
        self.config.token = None;
        self.http_clear_token();
    }

    pub fn ws_url_from_config(&self) -> Option<&Url> {
        self.config.ws_url.as_ref()
    }
}
