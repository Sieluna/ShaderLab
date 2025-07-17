use async_trait::async_trait;
use bytes::Bytes;

use crate::Result;

#[async_trait]
pub trait WsTransport {
    /// Send binary data through the WebSocket connection
    async fn send(&self, data: Bytes) -> Result<()>;

    /// Receive binary data from the WebSocket connection
    async fn receive(&self) -> Result<Bytes>;

    /// Establish WebSocket connection to the specified URL
    async fn connect(&self, url: &str) -> Result<()>;

    /// Check if the connection is currently active
    async fn is_connected(&self) -> bool;

    /// Close the WebSocket connection gracefully
    async fn close(&self) -> Result<()>;
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use std::sync::Arc;

    use futures_util::{SinkExt, StreamExt};
    use tokio::sync::{Mutex, RwLock, mpsc};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    use crate::{Error, WebSocketError};

    use super::*;

    pub struct NativeTransport {
        sender: Arc<Mutex<Option<mpsc::UnboundedSender<Bytes>>>>,
        receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<Bytes>>>>,
        connected: Arc<RwLock<bool>>,
    }

    impl NativeTransport {
        pub fn new() -> Self {
            Self {
                sender: Arc::new(Mutex::new(None)),
                receiver: Arc::new(Mutex::new(None)),
                connected: Arc::new(RwLock::new(false)),
            }
        }
    }

    #[async_trait]
    impl WsTransport for NativeTransport {
        async fn connect(&self, url: &str) -> Result<()> {
            // Check if already connected
            if *self.connected.read().await {
                return Ok(());
            }

            // Establish WebSocket connection
            let (ws_stream, _) = connect_async(url).await.map_err(|e| {
                Error::WebSocket(WebSocketError::ConnectionFailed {
                    message: e.to_string(),
                })
            })?;

            let (mut ws_sender, mut ws_receiver) = ws_stream.split();
            let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<Bytes>();
            let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<Bytes>();

            // Handle outgoing messages
            let connected_clone = Arc::clone(&self.connected);
            tokio::spawn(async move {
                while let Some(data) = outgoing_rx.recv().await {
                    if ws_sender.send(Message::Binary(data)).await.is_err() {
                        *connected_clone.write().await = false;
                        break;
                    }
                }
            });

            // Handle incoming messages
            let connected_clone = Arc::clone(&self.connected);
            tokio::spawn(async move {
                while let Some(msg) = ws_receiver.next().await {
                    match msg {
                        Ok(Message::Binary(data)) => {
                            if incoming_tx.send(data).is_err() {
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            *connected_clone.write().await = false;
                            break;
                        }
                        Err(_) => {
                            *connected_clone.write().await = false;
                            break;
                        }
                        _ => {} // Ignore other message types
                    }
                }
                *connected_clone.write().await = false;
            });

            // Store channels
            *self.sender.lock().await = Some(outgoing_tx);
            *self.receiver.lock().await = Some(incoming_rx);
            *self.connected.write().await = true;

            Ok(())
        }

        async fn send(&self, data: Bytes) -> Result<()> {
            if let Some(tx) = self.sender.lock().await.as_ref() {
                tx.send(data).map_err(|_| {
                    Error::WebSocket(WebSocketError::SendFailed {
                        message: "Channel closed".to_string(),
                    })
                })?;
                Ok(())
            } else {
                Err(Error::WebSocket(WebSocketError::NotConnected))
            }
        }

        async fn receive(&self) -> Result<Bytes> {
            if let Some(rx) = self.receiver.lock().await.as_mut() {
                rx.recv()
                    .await
                    .ok_or_else(|| Error::WebSocket(WebSocketError::ConnectionClosed))
            } else {
                Err(Error::WebSocket(WebSocketError::NotConnected))
            }
        }

        async fn is_connected(&self) -> bool {
            *self.connected.read().await
        }

        async fn close(&self) -> Result<()> {
            *self.connected.write().await = false;
            *self.sender.lock().await = None;
            *self.receiver.lock().await = None;
            Ok(())
        }
    }

    impl Default for NativeTransport {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use std::sync::{Arc, Mutex};

    use futures_channel::{mpsc::UnboundedReceiver, oneshot};
    use futures_util::StreamExt;
    use wasm_bindgen::{JsCast, prelude::*};
    use web_sys::{BinaryType, ErrorEvent, MessageEvent, WebSocket};

    use crate::{Error, WebSocketError};

    use super::*;

    pub struct WasmTransport {
        websocket: Arc<Mutex<Option<WebSocket>>>,
        receiver: Arc<Mutex<Option<UnboundedReceiver<Bytes>>>>,
        connected: Arc<Mutex<bool>>,
    }

    impl WasmTransport {
        pub fn new() -> Self {
            Self {
                websocket: Arc::new(Mutex::new(None)),
                receiver: Arc::new(Mutex::new(None)),
                connected: Arc::new(Mutex::new(false)),
            }
        }
    }

    // WASM is single-threaded, so it's safe to implement Send and Sync
    unsafe impl Send for WasmTransport {}
    unsafe impl Sync for WasmTransport {}

    #[async_trait]
    impl WsTransport for WasmTransport {
        async fn connect(&self, url: &str) -> Result<()> {
            // Check if already connected
            if *self.connected.lock().unwrap() {
                return Ok(());
            }

            // Create message channel
            let (tx, rx) = futures_channel::mpsc::unbounded::<Bytes>();
            *self.receiver.lock().unwrap() = Some(rx);

            // Connection completion channel
            let (conn_tx, conn_rx) = oneshot::channel::<std::result::Result<(), String>>();
            let conn_tx = Arc::new(Mutex::new(Some(conn_tx)));
            let connected = Arc::clone(&self.connected);

            // Setup and attach all handlers, then immediately forget them to avoid crossing await boundaries
            {
                // Create WebSocket
                let ws = WebSocket::new(url).map_err(|_| {
                    Error::WebSocket(WebSocketError::ConnectionFailed {
                        message: "Failed to create WebSocket".to_string(),
                    })
                })?;

                ws.set_binary_type(BinaryType::Arraybuffer);

                // Setup message handler
                let message_handler = {
                    let tx = tx.clone();
                    Closure::wrap(Box::new(move |e: MessageEvent| {
                        if let Ok(buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                            let array = js_sys::Uint8Array::new(&buffer);
                            let data = Bytes::from(array.to_vec());
                            let _ = tx.unbounded_send(data);
                        }
                    }) as Box<dyn FnMut(MessageEvent)>)
                };

                // Setup connection handlers
                let open_handler = {
                    let conn_tx = Arc::clone(&conn_tx);
                    let connected = Arc::clone(&connected);
                    Closure::wrap(Box::new(move |_| {
                        *connected.lock().unwrap() = true;
                        if let Some(tx) = conn_tx.lock().unwrap().take() {
                            let _ = tx.send(Ok(()));
                        }
                    }) as Box<dyn FnMut(JsValue)>)
                };

                let error_handler = {
                    let conn_tx = Arc::clone(&conn_tx);
                    Closure::wrap(Box::new(move |e: ErrorEvent| {
                        let error_msg = format!("WebSocket error: {:?}", e);
                        if let Some(tx) = conn_tx.lock().unwrap().take() {
                            let _ = tx.send(Err(error_msg));
                        }
                    }) as Box<dyn FnMut(ErrorEvent)>)
                };

                let close_handler = {
                    let connected = Arc::clone(&connected);
                    Closure::wrap(Box::new(move |_| {
                        *connected.lock().unwrap() = false;
                    }) as Box<dyn FnMut(JsValue)>)
                };

                // Attach handlers
                ws.set_onmessage(Some(message_handler.as_ref().unchecked_ref()));
                ws.set_onopen(Some(open_handler.as_ref().unchecked_ref()));
                ws.set_onerror(Some(error_handler.as_ref().unchecked_ref()));
                ws.set_onclose(Some(close_handler.as_ref().unchecked_ref()));

                // Store WebSocket
                *self.websocket.lock().unwrap() = Some(ws);

                // Prevent closure cleanup (must be done in this scope)
                message_handler.forget();
                open_handler.forget();
                error_handler.forget();
                close_handler.forget();
            } // All closures and WebSocket are now forgotten and out of scope

            // Wait for connection - no closures cross this await boundary
            let result = conn_rx.await;
            result
                .map_err(|_| {
                    Error::WebSocket(WebSocketError::ConnectionFailed {
                        message: "Connection interrupted".to_string(),
                    })
                })?
                .map_err(|e| Error::WebSocket(WebSocketError::ConnectionFailed { message: e }))
        }

        async fn send(&self, data: Bytes) -> Result<()> {
            if let Some(ws) = self.websocket.lock().unwrap().as_ref() {
                if *self.connected.lock().unwrap() {
                    ws.send_with_u8_array(&data).map_err(|_| {
                        Error::WebSocket(WebSocketError::SendFailed {
                            message: "Failed to send message".to_string(),
                        })
                    })?;
                    Ok(())
                } else {
                    Err(Error::WebSocket(WebSocketError::NotConnected))
                }
            } else {
                Err(Error::WebSocket(WebSocketError::NotConnected))
            }
        }

        async fn receive(&self) -> Result<Bytes> {
            // Take the receiver temporarily
            let rx = self.receiver.lock().unwrap().take();
            if let Some(mut receiver) = rx {
                // Get the next message
                let result = receiver
                    .next()
                    .await
                    .ok_or_else(|| Error::WebSocket(WebSocketError::ConnectionClosed));
                // Put the receiver back
                *self.receiver.lock().unwrap() = Some(receiver);
                result
            } else {
                Err(Error::WebSocket(WebSocketError::NotConnected))
            }
        }

        async fn is_connected(&self) -> bool {
            *self.connected.lock().unwrap()
        }

        async fn close(&self) -> Result<()> {
            if let Some(ws) = self.websocket.lock().unwrap().as_ref() {
                ws.close().map_err(|_| {
                    Error::WebSocket(WebSocketError::SendFailed {
                        message: "Failed to close WebSocket".to_string(),
                    })
                })?;
            }
            *self.connected.lock().unwrap() = false;
            *self.websocket.lock().unwrap() = None;
            *self.receiver.lock().unwrap() = None;
            Ok(())
        }
    }

    impl Default for WasmTransport {
        fn default() -> Self {
            Self::new()
        }
    }
}
