use std::pin::Pin;
use std::sync::Arc;

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{futures, stream};
use senra_api::{Endpoint, Request};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message as WsMessage, Utf8Bytes};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::{Message, NetworkError, NetworkInner};

impl From<tokio_tungstenite::tungstenite::Error> for NetworkError {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        NetworkError::WebSocket(error.to_string())
    }
}

impl From<reqwest::Error> for NetworkError {
    fn from(error: reqwest::Error) -> Self {
        NetworkError::Http(error.to_string())
    }
}

#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connected(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        mpsc::Receiver<Utf8Bytes>,
    ),
}

pub struct NativeNetwork {
    state: Arc<Mutex<ConnectionState>>,
    client: reqwest::Client,
}

impl NativeNetwork {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            client: reqwest::Client::new(),
        }
    }

    fn get_headers(&self, token: Option<&str>) -> http::HeaderMap {
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
        headers
    }
}

#[async_trait::async_trait]
impl NetworkInner for NativeNetwork {
    fn subscription(&self) -> Pin<Box<dyn Stream<Item = Message> + Send>> {
        let state = self.state.clone();
        Box::pin(stream::channel(100, move |mut output| async move {
            loop {
                let mut state = state.lock().await;
                match &mut *state {
                    ConnectionState::Connected(websocket, input) => {
                        let mut fused_websocket = websocket.by_ref().fuse();

                        futures::select! {
                            received = fused_websocket.select_next_some() => {
                                match received {
                                    Ok(WsMessage::Text(message)) => {
                                        output.send(
                                            match serde_json::from_str(message.as_str()) {
                                                Ok(response) => Message::Incoming(response),
                                                Err(e) => Message::Error(NetworkError::Serialization(e).to_string()),
                                            }
                                        ).await.unwrap();
                                    }
                                    Err(e) => {
                                        output.send(Message::Error(NetworkError::WebSocket(e.to_string()).to_string())).await.unwrap();
                                        output.send(Message::Disconnected).await.unwrap();
                                        *state = ConnectionState::Disconnected;
                                    }
                                    Ok(_) => continue,
                                }
                            }

                            message = input.select_next_some() => {
                                let result = websocket.send(WsMessage::Text(message)).await;

                                if let Err(e) = result {
                                    output.send(Message::Error(NetworkError::WebSocket(e.to_string()).to_string())).await.unwrap();
                                    output.send(Message::Disconnected).await.unwrap();
                                    *state = ConnectionState::Disconnected;
                                }
                            }
                        }
                    }
                    _ => break,
                }
            }
        }))
    }

    async fn connect(&self, url: &str, token: Option<&str>) -> Result<Message, NetworkError> {
        let mut request = url.into_client_request()?;
        request.headers_mut().extend(self.get_headers(token));
        let (websocket, _) = connect_async(request).await?;

        let (sender, receiver) = mpsc::channel(100);
        *self.state.lock().await = ConnectionState::Connected(websocket, receiver);
        Ok(Message::Connected(sender))
    }

    async fn fetch(
        &self,
        url: &str,
        token: Option<&str>,
        request: Request,
    ) -> Result<Message, NetworkError> {
        let endpoint: Endpoint = request.to_owned().into();
        let url = format!("{}{}", url, endpoint.path);
        let response = self
            .client
            .request(endpoint.method, &url)
            .headers(self.get_headers(token))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let response: senra_api::Response = response.json().await?;
            Ok(Message::Incoming(response))
        } else {
            let error = response.text().await?;
            Ok(Message::Error(error))
        }
    }
}
