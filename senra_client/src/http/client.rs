use reqwest::{Client, Request, Response};
use senra_api::Codec;
use serde::{Deserialize, Serialize};

use super::error::{Error, Result};
use crate::config::ClientConfig;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    config: ClientConfig,
}

impl HttpClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let mut builder = Client::builder();

        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder.timeout(std::time::Duration::from_millis(config.timeout_ms));
        }

        let client = builder.build()?;
        Ok(Self { client, config })
    }

    pub async fn request<Req, Res>(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<Req>,
    ) -> Result<Res>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let req = self.build_request(method, endpoint, body)?;
        let response = self.client.execute(req).await?;
        self.handle_response(response).await
    }

    pub fn base_url(&self) -> &str {
        self.config.http_url.as_str()
    }

    pub fn token(&self) -> Option<&str> {
        self.config.token.as_deref()
    }

    pub fn set_token(&mut self, token: String) {
        self.config.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.config.token = None;
    }

    fn build_request<T: Serialize>(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<T>,
    ) -> Result<Request> {
        let base = self.config.http_url.as_str().trim_end_matches('/');
        let endpoint = endpoint.trim_start_matches('/');
        let url = format!("{}/{}", base, endpoint);

        let mut req_builder = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "PATCH" => self.client.patch(&url),
            "DELETE" => self.client.delete(&url),
            m => return Err(Error::UnsupportedMethod(m.to_string())),
        };

        if let Some(token) = &self.config.token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        req_builder = req_builder.header(
            "Content-Type",
            self.config.protocol_config.codec.content_type(),
        );

        if let Some(data) = body {
            let bytes = self.config.protocol_config.codec.encode(&data)?;
            req_builder = req_builder.body(bytes);
        }

        Ok(req_builder.build()?)
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let body = response.bytes().await?;

        if status.is_success() {
            Ok(self.config.protocol_config.codec.decode(&body)?)
        } else {
            Err(Error::Status {
                status: status.as_u16(),
                message: String::from_utf8_lossy(&body).to_string(),
            })
        }
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for HttpClient {}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for HttpClient {}
