#[cfg(not(target_arch = "wasm32"))]
mod native;
// #[cfg(target_arch = "wasm32")]
// mod web;

use std::pin::Pin;
use std::sync::Arc;

use http::Method;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{Subscription, Task};
use senra_api::{Request, Response};
use tokio_tungstenite::tungstenite::Utf8Bytes;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Http(Method),
    WebSocket,
}

#[derive(Debug, Clone)]
pub enum NetworkMessage {
    Incoming(Response),
    Outgoing(Protocol, Request),
    Connected(mpsc::Sender<Utf8Bytes>),
    Disconnected,
    Error(String),
}

#[async_trait::async_trait]
pub trait NetworkInner: Send + Sync {
    fn subscription(&self) -> Pin<Box<dyn Stream<Item = NetworkMessage> + Send>>;

    async fn connect(&self, url: &str, token: &str) -> Result<NetworkMessage, NetworkError>;

    async fn fetch(
        &self,
        url: &str,
        method: Method,
        request: Request,
    ) -> Result<NetworkMessage, NetworkError>;
}

pub struct Network {
    inner: Arc<dyn NetworkInner>,
    sender: Option<mpsc::Sender<Utf8Bytes>>,
    base_url: Arc<str>,
}

impl Network {
    pub fn new(base_url: Arc<str>) -> Self {
        let base_url = Arc::clone(&base_url);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let network = native::NativeNetwork::new();
            Self {
                inner: Arc::new(network),
                sender: None,
                base_url,
            }
        }
    }

    pub fn update(&mut self, message: NetworkMessage) -> Task<NetworkMessage> {
        match message {
            NetworkMessage::Connected(sender) => {
                self.sender = Some(sender);
                Task::none()
            }
            NetworkMessage::Outgoing(protocol, request) => match protocol {
                Protocol::Http(method) => self.handle_http(method, request),
                Protocol::WebSocket => self.handle_websocket(request),
            },
            _ => Task::none(),
        }
    }

    pub fn subscribe(&self) -> Subscription<NetworkMessage> {
        Subscription::run_with_id(stringify!(Transport), self.inner.clone().subscription())
    }

    fn handle_http(&self, method: Method, request: Request) -> Task<NetworkMessage> {
        let inner = self.inner.clone();
        let base_url = self.base_url.clone();
        Task::perform(
            async move { inner.fetch(&base_url, method, request).await },
            |result| result.unwrap_or_else(|e| NetworkMessage::Error(e.to_string())),
        )
    }

    fn handle_websocket(&self, request: Request) -> Task<NetworkMessage> {
        match self.sender.clone() {
            Some(mut sender) => Task::perform(
                async move {
                    match serde_json::to_string(&request) {
                        Ok(message) => {
                            if let Err(e) = sender.send(message.into()).await {
                                return Err(format!("Failed to send message: {}", e));
                            }
                            Ok(request)
                        }
                        Err(e) => Err(format!("Failed to serialize message: {}", e)),
                    }
                },
                |result| match result {
                    Ok(request) => NetworkMessage::Outgoing(Protocol::WebSocket, request),
                    Err(e) => NetworkMessage::Error(e),
                },
            ),
            None => Task::done(NetworkMessage::Error("Not connected".to_string())),
        }
    }
}
