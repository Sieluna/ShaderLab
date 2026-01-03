mod stream;

pub use stream::{WsState, WsStream};

use std::sync::Arc;

use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{BinaryType, WebSocket as WebSysSocket};

use crate::ws::error::{Error, Result};

pub struct WebSocket {
    ws: Arc<WebSysSocket>,
}

impl WebSocket {
    pub async fn connect(url: &Url) -> Result<(Self, WsStream)> {
        let ws = Arc::new(
            WebSysSocket::new(url.as_str())
                .map_err(|_| Error::Transport(format!("failed to create websocket: {}", url)))?,
        );

        ws.set_binary_type(BinaryType::Arraybuffer);

        let ws_clone = Arc::clone(&ws);
        let (tx, rx) = futures_channel::oneshot::channel();
        let tx = std::cell::RefCell::new(Some(tx));

        {
            let tx = &tx;
            let ws_ref = &ws;

            let on_open = Closure::wrap(Box::new(move || {
                if let Ok(mut tx_guard) = tx.try_borrow_mut() {
                    if let Some(tx) = tx_guard.take() {
                        let _ = tx.send(Ok(()));
                    }
                }
            }) as Box<dyn FnMut()>);

            let on_error = {
                let tx = &tx;
                Closure::wrap(Box::new(move |_| {
                    if let Ok(mut tx_guard) = tx.try_borrow_mut() {
                        if let Some(tx) = tx_guard.take() {
                            let _ = tx.send(Err(Error::Transport("connection error".to_string())));
                        }
                    }
                }) as Box<dyn FnMut(web_sys::Event)>)
            };

            ws_ref.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            ws_ref.set_onerror(Some(on_error.as_ref().unchecked_ref()));

            on_open.forget();
            on_error.forget();
        }

        rx.await
            .map_err(|_| Error::Transport("connection interrupted".to_string()))??;

        Ok((Self { ws: ws_clone }, WsStream::new(ws)))
    }

    pub fn url(&self) -> String {
        self.ws.url()
    }
}

pub async fn connect(url: &Url) -> Result<(WebSocket, WsStream)> {
    WebSocket::connect(url).await
}
