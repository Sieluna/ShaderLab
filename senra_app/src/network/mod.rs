#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use std::pin::Pin;
use std::sync::Arc;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream};
use iced::{Subscription, Task};
use senra_api::{ApiError, Client, Request, Response};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(#[from] ApiError),
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
    MessageRequest(Protocol, Request),
    ConnectRequest(String),

    MessageRespond(Response),
    MessageSubmit,

    Connect(mpsc::Sender<String>),
    Disconnect,

    Error(String),
}

#[async_trait::async_trait]
pub trait NetworkInner: Send + Sync {
    fn subscription(&self) -> Pin<Box<dyn Stream<Item = Message> + Send>>;

    async fn connect(&self, url: &str) -> Result<Message, NetworkError>;
}

#[derive(Clone)]
pub struct Network {
    inner: Arc<dyn NetworkInner>,
    sender: Option<mpsc::Sender<String>>,
    client: Client,
}

impl Network {
    pub fn new(config: &Config) -> Self {
        let network = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                native::NativeNetwork::new()
            }
            #[cfg(target_arch = "wasm32")]
            {
                web::WebNetwork::new()
            }
        };

        Self {
            inner: Arc::new(network),
            client: Client::new(config.url.clone()),
            sender: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MessageRequest(protocol, request) => match protocol {
                Protocol::Http => self.handle_http(request),
                Protocol::WebSocket => self.handle_websocket(request),
            },
            Message::ConnectRequest(token) => {
                let url = format!(
                    "{}/ws?token={}",
                    self.client.url().replace("http", "ws"),
                    &token
                );
                let inner = self.inner.clone();
                self.client.set_token(token);
                Task::perform(async move { inner.connect(url.as_ref()).await }, |result| {
                    result.unwrap_or_else(|e| Message::Error(e.to_string()))
                })
            }
            Message::Connect(sender) => {
                self.sender = Some(sender);
                Task::none()
            }
            Message::Disconnect => {
                self.sender = None;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn subscribe(&self) -> Subscription<Message> {
        Subscription::run_with_id(stringify!(Transport), self.inner.clone().subscription())
    }

    fn handle_http(&self, request: Request) -> Task<Message> {
        let client = self.client.clone();

        Task::perform(
            async move {
                match client.request(request).await {
                    Ok(response) => Ok(Message::MessageRespond(response)),
                    Err(e) => Err(NetworkError::Api(e)),
                }
            },
            |result| result.unwrap_or_else(|e: NetworkError| Message::Error(e.to_string())),
        )
    }

    fn handle_websocket(&self, request: Request) -> Task<Message> {
        match self.sender.clone() {
            Some(mut sender) => Task::perform(
                async move {
                    let message = serde_json::to_string(&request)?;

                    sender
                        .send(message)
                        .await
                        .map_err(|e| NetworkError::WebSocket(e.to_string()))?;

                    Ok(Message::MessageSubmit)
                },
                |result| result.unwrap_or_else(|e: NetworkError| Message::Error(e.to_string())),
            ),
            None => Task::done(Message::Error("Not connected".to_string())),
        }
    }
}
