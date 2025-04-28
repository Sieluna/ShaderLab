use senra_api::{AsyncResult, Codec, Pipeline, PipelineContext, ProtocolError};
use serde::{Deserialize, Serialize};

use crate::{ClientConfig, Error, Result};

#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    config: ClientConfig,
}

impl HttpClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let builder = reqwest::Client::builder();

        #[cfg(not(target_arch = "wasm32"))]
        let builder = if let Some(timeout_ms) = config.timeout_ms {
            builder.timeout(core::time::Duration::from_millis(timeout_ms))
        } else {
            builder
        };

        let client = builder.build().map_err(|e| Error::Network(e.to_string()))?;

        Ok(Self { client, config })
    }

    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    pub fn token(&self) -> Option<&str> {
        self.config.token.as_deref()
    }
}

#[cfg(target_arch = "wasm32")]
unsafe impl Send for HttpClient {}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for HttpClient {}

impl HttpClient {
    async fn build_request<T: Serialize>(
        &self,
        method: &str,
        endpoint: &str,
        request: Option<T>,
    ) -> Result<reqwest::Request> {
        let url = format!("{}{}", self.config.base_url, endpoint);

        let mut req_builder = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "PATCH" => self.client.patch(&url),
            "DELETE" => self.client.delete(&url),
            _ => {
                return Err(Error::BadRequest(format!("Unsupported method: {}", method)));
            }
        };

        if let Some(token) = &self.config.token {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", token));
        }

        req_builder = req_builder.header(
            "Content-Type",
            self.config.protocol_config.codec.content_type(),
        );

        if let Some(request_data) = request {
            let body = self.config.protocol_config.codec.encode(&request_data)?;
            req_builder = req_builder.body(body);
        }

        req_builder
            .build()
            .map_err(|e| Error::Network(e.to_string()))
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        match status.as_u16() {
            200..=299 => self
                .config
                .protocol_config
                .codec
                .decode(&body)
                .map_err(Error::from),
            401 => Err(Error::Authentication("Unauthorized".to_string())),
            404 => Err(Error::NotFound("Resource not found".to_string())),
            400..=499 => Err(Error::BadRequest(
                String::from_utf8_lossy(&body).to_string(),
            )),
            500..=599 => Err(Error::InternalServerError(
                String::from_utf8_lossy(&body).to_string(),
            )),
            _ => Err(Error::Unknown(format!("Unexpected status: {}", status))),
        }
    }

    pub async fn request<Req, Res>(
        &self,
        method: &str,
        endpoint: &str,
        request: Option<Req>,
    ) -> Result<Res>
    where
        Req: Serialize,
        Res: for<'de> Deserialize<'de>,
    {
        let req = self.build_request(method, endpoint, request).await?;
        let response = self
            .client
            .execute(req)
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        self.handle_response(response).await
    }
}

pub struct HttpProtocol;

impl<Req, Res> Pipeline<Req, Res> for HttpClient
where
    Req: Serialize + for<'de> Deserialize<'de> + Send + 'static,
    Res: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    type Protocol = HttpProtocol;

    fn execute(&self, request: Req, context: PipelineContext) -> AsyncResult<Res> {
        let client = self.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            Box::pin(async move {
                let endpoint = context.metadata.get("endpoint").cloned().ok_or_else(|| {
                    use senra_api::ProtocolError;

                    ProtocolError::Unknown("Missing endpoint in context".to_string())
                })?;
                let method = context
                    .metadata
                    .get("method")
                    .cloned()
                    .unwrap_or_else(|| "POST".to_string());

                client
                    .request(&method, &endpoint, Some(request))
                    .await
                    .map_err(|e| ProtocolError::TransportError(e.to_string()))
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            use futures_channel::oneshot;
            use wasm_bindgen_futures::spawn_local;

            let (tx, rx) = oneshot::channel();

            spawn_local(async move {
                let result = async {
                    let endpoint = context.metadata.get("endpoint").cloned().ok_or_else(|| {
                        ProtocolError::Unknown("Missing endpoint in context".to_string())
                    })?;
                    let method = context
                        .metadata
                        .get("method")
                        .cloned()
                        .unwrap_or_else(|| "POST".to_string());

                    client
                        .request(&method, &endpoint, Some(request))
                        .await
                        .map_err(|e| ProtocolError::TransportError(e.to_string()))
                }
                .await;

                let _ = tx.send(result);
            });

            Box::pin(async move {
                rx.await
                    .map_err(|_| ProtocolError::Unknown("Channel closed".to_string()))?
            })
        }
    }
}
