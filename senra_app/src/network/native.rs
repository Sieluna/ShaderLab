use std::pin::Pin;
use std::sync::Arc;

use iced::futures::channel::mpsc;
use iced::futures::lock::Mutex;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{futures, stream};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::{Message, NetworkError, NetworkInner};

impl From<tokio_tungstenite::tungstenite::Error> for NetworkError {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        NetworkError::WebSocket(error.to_string())
    }
}

#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connected(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        mpsc::Receiver<String>,
    ),
}

pub struct NativeNetwork {
    state: Arc<Mutex<ConnectionState>>,
}

impl NativeNetwork {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
        }
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
                                let result = websocket.send(WsMessage::text(message)).await;

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

    async fn connect(&self, url: &str) -> Result<Message, NetworkError> {
        let request = url.into_client_request()?;
        let (websocket, _) = connect_async(request).await?;

        let (sender, receiver) = mpsc::channel(100);
        *self.state.lock().await = ConnectionState::Connected(websocket, receiver);
        Ok(Message::Connected(sender))
    }
}
