use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use senra_api::*;
use serde::{Deserialize, Serialize};

use crate::{ClientConfig, Error, Result, WebSocketError};

use super::{WsMessage, WsResponse, WsTransport};

#[cfg(not(target_arch = "wasm32"))]
use super::transport::native::NativeTransport;

#[cfg(target_arch = "wasm32")]
use super::transport::wasm::WasmTransport;

pub struct WsClient {
    config: ClientConfig,
    #[cfg(not(target_arch = "wasm32"))]
    transport: Arc<NativeTransport>,
    #[cfg(target_arch = "wasm32")]
    transport: Arc<WasmTransport>,
}

impl WsClient {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        let transport = Arc::new(NativeTransport::new());

        #[cfg(target_arch = "wasm32")]
        let transport = Arc::new(WasmTransport::new());

        Ok(Self { config, transport })
    }

    /// Establish WebSocket connection to the server
    pub async fn connect(&self) -> Result<()> {
        let ws_url = self.build_websocket_url()?;
        self.transport.connect(&ws_url).await
    }

    /// Check if the WebSocket connection is active
    pub async fn is_connected(&self) -> bool {
        self.transport.is_connected().await
    }

    /// Close the WebSocket connection gracefully
    pub async fn close(&self) -> Result<()> {
        self.transport.close().await
    }

    /// Send raw data through the WebSocket connection
    pub async fn send<T: Serialize>(&self, data: &T) -> Result<()> {
        self.ensure_connected().await?;
        let encoded = self.config.protocol_config.codec.encode(data)?;
        self.transport.send(Bytes::from(encoded)).await
    }

    /// Receive and decode data from the WebSocket connection
    pub async fn receive<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        let data = self.transport.receive().await?;
        self.config
            .protocol_config
            .codec
            .decode(&data)
            .map_err(Error::from)
    }

    /// Send a structured message with automatic ID generation and timestamping
    pub async fn send_message<T: Serialize>(&self, message_type: &str, payload: T) -> Result<()> {
        let message = WsMessage {
            id: self.generate_message_id(),
            message_type: message_type.to_string(),
            payload,
            timestamp: self.current_timestamp(),
        };

        self.send(&message).await
    }

    /// Receive a structured response message
    pub async fn receive_response<T: for<'de> Deserialize<'de>>(&self) -> Result<WsResponse<T>> {
        self.receive().await
    }

    /// Send a request and wait for a response (request-response pattern)
    pub async fn request<Req: Serialize, Res: for<'de> Deserialize<'de>>(
        &self,
        message_type: &str,
        payload: Req,
    ) -> Result<WsResponse<Res>> {
        self.send_message(message_type, payload).await?;
        self.receive_response().await
    }

    /// Build WebSocket URL from the base configuration
    fn build_websocket_url(&self) -> Result<String> {
        let token = self.config.token.as_ref().ok_or_else(|| Error::Auth {
            message: "Token required for WebSocket connection".to_string(),
        })?;

        Ok(format!(
            "{}{}?token={}",
            self.config.base_url.replace("http", "ws"),
            "/ws",
            token
        ))
    }

    /// Ensure the connection is active before performing operations
    async fn ensure_connected(&self) -> Result<()> {
        if !self.is_connected().await {
            return Err(Error::WebSocket(WebSocketError::NotConnected));
        }
        Ok(())
    }

    /// Generate a unique message ID based on current timestamp
    fn generate_message_id(&self) -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string()
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

impl<Req, Res> Pipeline<Req, Res> for WsClient
where
    Req: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
    Res: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    type Protocol = WsProtocol;

    fn execute(&self, request: Req, _context: PipelineContext) -> AsyncResult<Res> {
        let client = self.clone();

        #[cfg(not(target_arch = "wasm32"))]
        {
            Box::pin(async move {
                client
                    .send(&request)
                    .await
                    .map_err(|e| ProtocolError::Transport {
                        message: e.to_string(),
                    })?;

                let response = client
                    .receive()
                    .await
                    .map_err(|e| ProtocolError::Transport {
                        message: e.to_string(),
                    })?;

                Ok(response)
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            use futures_channel::oneshot;
            use wasm_bindgen_futures::spawn_local;

            let (tx, rx) = oneshot::channel();

            spawn_local(async move {
                let result = async {
                    client
                        .send(&request)
                        .await
                        .map_err(|e| ProtocolError::Transport {
                            message: e.to_string(),
                        })?;

                    let response =
                        client
                            .receive()
                            .await
                            .map_err(|e| ProtocolError::Transport {
                                message: e.to_string(),
                            })?;

                    Ok(response)
                }
                .await;

                let _ = tx.send(result);
            });

            Box::pin(async move {
                rx.await.map_err(|_| ProtocolError::Unknown {
                    message: "Channel closed".to_string(),
                })?
            })
        }
    }
}

impl Clone for WsClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            transport: Arc::clone(&self.transport),
        }
    }
}

/// WebSocket protocol marker
pub struct WsProtocol;

#[cfg(target_arch = "wasm32")]
unsafe impl Send for WsClient {}

#[cfg(target_arch = "wasm32")]
unsafe impl Sync for WsClient {}
