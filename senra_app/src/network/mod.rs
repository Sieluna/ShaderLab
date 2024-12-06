#[cfg(not(target_arch = "wasm32"))]
mod native;
// #[cfg(target_arch = "wasm32")]
// mod web;

use std::pin::Pin;
use std::sync::Arc;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream};
use iced::{Subscription, Task};
use senra_api::{Request, Response};
use tokio_tungstenite::tungstenite::Utf8Bytes;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
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
    async fn subscription(&mut self) -> Pin<Box<dyn Stream<Item = NetworkMessage> + Send>>;

    async fn connect(&mut self, url: &str, token: &str) -> Result<NetworkMessage, NetworkError>;

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

    pub fn subscribe(&mut self) -> Subscription<NetworkMessage> {
        Subscription::run_with_id(stringify!(Transport), self.inner.stream())
    }

    fn handle_http(&mut self, method: Method, request: Request) -> Task<NetworkMessage> {
        self.inner.fetch("", method, request)
    }

    fn handle_websocket(&mut self, request: Request) -> Task<NetworkMessage> {
        match self.sender.clone() {
            Some(mut sender) => Task::perform(
                async move {
                    match serde_json::to_string(&request) {
                        Ok(message) => {
                            if let Err(e) = sender.send(message.into()).await {
                                return Err(format!("Failed to send message: {}", e));
                            } else {
                                Ok(())
                            }
                        }
                        Err(e) => Err(format!("Failed to serialize message: {}", e)),
                    }
                },
                |result| result.err().map(NetworkMessage::Error),
            )
            .then(|opt| opt.map(Task::done).unwrap_or_else(Task::none)),
            None => Task::none(),
        }
    }
}
