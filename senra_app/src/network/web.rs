use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use iced::futures::channel::{mpsc, oneshot};
use iced::futures::{Stream, StreamExt};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use super::{Message, NetworkError, NetworkInner};

impl From<JsValue> for NetworkError {
    fn from(error: JsValue) -> Self {
        NetworkError::WebSocket(format!("{:?}", error))
    }
}

pub struct WebNetwork {
    event_tx: Rc<RefCell<Option<mpsc::UnboundedSender<Message>>>>,
}

impl WebNetwork {
    pub fn new() -> Self {
        Self {
            event_tx: Rc::new(RefCell::new(None)),
        }
    }
}

unsafe impl Send for WebNetwork {}
unsafe impl Sync for WebNetwork {}

#[async_trait::async_trait]
impl NetworkInner for WebNetwork {
    fn subscription(&self) -> Pin<Box<dyn Stream<Item = Message> + Send>> {
        let (tx, rx) = mpsc::unbounded();

        *self.event_tx.borrow_mut() = Some(tx);
        
        Box::pin(rx)
    }

    async fn connect(&self, url: &str) -> Result<Message, NetworkError> {
        let ws = WebSocket::new(url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let (cmd_tx, mut cmd_rx) = mpsc::channel(100);
        let event_tx = self.event_tx.clone();

        {
            let event_tx = event_tx.clone();
            let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    if let Some(tx) = event_tx.borrow_mut().as_mut() {
                        if let Ok(response) = serde_json::from_str(&text) {
                            let _ = tx.unbounded_send(Message::Incoming(response));
                        }
                    }
                }
            });
            ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();
        }

        {
            let event_tx = event_tx.clone();
            let onerror = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                if let Some(tx) = event_tx.borrow_mut().as_mut() {
                    let _ = tx.unbounded_send(Message::Error(format!("WS错误: {:?}", e)));
                }
            });
            ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();
        }

        {
            let event_tx = event_tx.clone();
            let onclose = Closure::<dyn FnMut()>::new(move || {
                if let Some(tx) = event_tx.borrow_mut().as_mut() {
                    let _ = tx.unbounded_send(Message::Disconnected);
                }
            });
            ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
            onclose.forget();
        }

        {
            let event_tx = event_tx.clone();
            let cmd_tx_clone = cmd_tx.clone();
            let onopen = Closure::<dyn FnMut()>::new(move || {
                if let Some(tx) = event_tx.borrow_mut().as_mut() {
                    let _ = tx.unbounded_send(Message::Connected(cmd_tx_clone.clone()));
                }
            });
            ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            onopen.forget();
        }

        let ws_clone = ws.clone();
        spawn_local(async move {
            while let Some(msg) = cmd_rx.next().await {
                if let Err(e) = ws_clone.send_with_str(&msg) {
                    if let Some(tx) = event_tx.borrow_mut().as_mut() {
                        let _ = tx.unbounded_send(Message::Error(format!("发送失败: {:?}", e)));
                    }
                }
            }
        });

        Ok(Message::Connected(cmd_tx))
    }
}
