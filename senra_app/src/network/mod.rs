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
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Http,
    WebSocket,
}

#[derive(Debug, Clone)]
pub enum Message {
    AuthToken(String),
    Outgoing(Protocol, Request),

    Incoming(Response),
    Connected(mpsc::Sender<Utf8Bytes>),
    Disconnected,
    Submitted,

    Error(String),
}

#[async_trait::async_trait]
pub trait NetworkInner: Send + Sync {
    fn subscription(&self) -> Pin<Box<dyn Stream<Item = Message> + Send>>;

    async fn connect(&self, url: &str, token: Option<&str>) -> Result<Message, NetworkError>;

    async fn fetch(
        &self,
        url: &str,
        token: Option<&str>,
        request: Request,
    ) -> Result<Message, NetworkError>;
}

#[derive(Clone)]
pub struct Network {
    inner: Arc<dyn NetworkInner>,
    sender: Option<mpsc::Sender<Utf8Bytes>>,
    base_url: String,
    auth_token: Option<String>,
}

impl Network {
    pub fn new(base_url: String) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let network = native::NativeNetwork::new();
            Self {
                inner: Arc::new(network),
                sender: None,
                base_url,
                auth_token: None,
            }
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AuthToken(token) => {
                self.auth_token = Some(token);
                let inner = self.inner.clone();
                let url = self.base_url.clone();
                let token = self.auth_token.clone();
                Task::perform(
                    async move { inner.connect(url.as_ref(), token.as_deref()).await },
                    |result| result.unwrap_or_else(|e| Message::Error(e.to_string())),
                )
            }
            Message::Outgoing(protocol, request) => match protocol {
                Protocol::Http => self.handle_http(request),
                Protocol::WebSocket => self.handle_websocket(request),
            },
            Message::Connected(sender) => {
                self.sender = Some(sender);
                Task::none()
            },
            _ => Task::none(),
        }
    }

    pub fn subscribe(&self) -> Subscription<Message> {
        Subscription::run_with_id(stringify!(Transport), self.inner.clone().subscription())
    }

    fn handle_http(&self, request: Request) -> Task<Message> {
        let inner = self.inner.clone();
        let url = self.base_url.clone();
        let token = self.auth_token.clone();
        Task::perform(
            async move { inner.fetch(url.as_ref(), token.as_deref(), request).await },
            |result| result.unwrap_or_else(|e| Message::Error(e.to_string())),
        )
    }

    fn handle_websocket(&self, request: Request) -> Task<Message> {
        match self.sender.clone() {
            Some(mut sender) => Task::perform(
                async move {
                    let message = serde_json::to_string(&request)?;

                    sender
                        .send(message.into())
                        .await
                        .map_err(|e| NetworkError::WebSocket(e.to_string()))?;

                    Ok(Message::Submitted)
                },
                |result| result.unwrap_or_else(|e: NetworkError| Message::Error(e.to_string())),
            ),
            None => Task::done(Message::Error("Not connected".to_string())),
        }
    }
}
