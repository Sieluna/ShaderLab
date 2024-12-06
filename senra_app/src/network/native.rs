use std::pin::Pin;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{futures, stream};

use senra_api::Request;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::{Method, NetworkError, NetworkInner, NetworkMessage};

#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connected(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        mpsc::Receiver<Utf8Bytes>,
    ),
}

pub struct NativeNetwork {
    state: ConnectionState,
}

impl NativeNetwork {
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
        }
    }
}

#[async_trait::async_trait]
impl NetworkInner for NativeNetwork {
    async fn subscription(&mut self) -> Pin<Box<dyn Stream<Item = NetworkMessage> + Send>> {
        Box::pin(stream::channel(100, move |mut output| async move {
            loop {
                match &mut self.state {
                    ConnectionState::Connected(websocket, input) => {
                        let mut fused_websocket = websocket.by_ref().fuse();

                        futures::select! {
                            received = fused_websocket.select_next_some() => {
                                match received {
                                    Ok(Message::Text(message)) => {
                                        output.send(
                                            match serde_json::from_str(message.as_str()) {
                                                Ok(response) => NetworkMessage::Incoming(response),
                                                Err(e) => NetworkMessage::Error(format!("Failed to parse message: {}", e)),
                                            }
                                        ).await.unwrap();
                                    }
                                    Err(_) => {
                                        output.send(NetworkMessage::Disconnected).await.unwrap();
                                        self.state = ConnectionState::Disconnected;
                                    }
                                    Ok(_) => continue,
                                }
                            }

                            message = input.select_next_some() => {
                                let result = websocket.send(Message::Text(message)).await;

                                if result.is_err() {
                                    output.send(NetworkMessage::Disconnected).await.unwrap();
                                    self.state = ConnectionState::Disconnected;
                                }
                            }
                        }
                    }
                    _ => break,
                }
            }
        }))
    }

    async fn connect(&mut self, url: &str, token: &str) -> Result<NetworkMessage, NetworkError> {
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        match connect_async(request).await {
            Ok((websocket, _)) => {
                let (sender, receiver) = mpsc::channel(100);

                self.state = ConnectionState::Connected(websocket, receiver);

                Ok(NetworkMessage::Connected(sender))
            }
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Ok(NetworkMessage::Disconnected)
            }
        }
    }

    async fn fetch(
        &self,
        url: &str,
        method: Method,
        request: Request,
    ) -> Result<NetworkMessage, NetworkError> {
        todo!()
    }
}
