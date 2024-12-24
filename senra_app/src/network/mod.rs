#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use std::pin::Pin;
use std::sync::Arc;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream};
use iced::{Subscription, Task};
use senra_api::{Endpoint, Request, Response};

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
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
    client: reqwest::Client,
    sender: Option<mpsc::Sender<String>>,
    base_url: String,
    auth_token: Option<String>,
}

impl Network {
    pub fn new(base_url: String) -> Self {
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
            client: reqwest::Client::new(),
            sender: None,
            base_url,
            auth_token: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MessageRequest(protocol, request) => match protocol {
                Protocol::Http => self.handle_http(request),
                Protocol::WebSocket => self.handle_websocket(request),
            },
            Message::ConnectRequest(token) => {
                let url = format!("{}/ws?token={}", &self.base_url, &token);
                let inner = self.inner.clone();
                self.auth_token = Some(token);
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
        let url = self.base_url.clone();
        let token = self.auth_token.clone();

        Task::perform(
            async move {
                let endpoint: Endpoint = request.to_owned().into();
                let url = format!("{}{}", url, endpoint.path);
                let mut headers = http::HeaderMap::new();
                headers.insert(
                    http::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                if let Some(token) = token {
                    headers.insert(
                        http::header::AUTHORIZATION,
                        format!("Bearer {}", token).parse().unwrap(),
                    );
                }
                let response = client
                    .request(endpoint.method, &url)
                    .headers(headers)
                    .json(&request)
                    .send()
                    .await?;

                if response.status().is_success() {
                    let response: Response = response.json().await?;
                    Ok(Message::MessageRespond(response))
                } else {
                    let error = response.text().await?;
                    Ok(Message::Error(error))
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
