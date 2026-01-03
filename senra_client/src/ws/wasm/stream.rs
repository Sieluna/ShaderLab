use core::cell::RefCell;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use std::collections::VecDeque;
use std::sync::Arc;

use futures_util::{Sink, Stream};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{CloseEvent as JsCloseEvt, MessageEvent, WebSocket as WebSysSocket};

use crate::ws::Message;
use crate::ws::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsState {
    Connecting,
    Open,
    Closing,
    Closed,
}

impl TryFrom<u16> for WsState {
    type Error = Error;

    fn try_from(state: u16) -> Result<Self> {
        match state {
            WebSysSocket::CONNECTING => Ok(WsState::Connecting),
            WebSysSocket::OPEN => Ok(WsState::Open),
            WebSysSocket::CLOSING => Ok(WsState::Closing),
            WebSysSocket::CLOSED => Ok(WsState::Closed),
            _ => Err(Error::Transport(format!("invalid ws state: {}", state))),
        }
    }
}

impl WsState {
    pub fn is_open(&self) -> bool {
        matches!(self, WsState::Open)
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self, WsState::Connecting)
    }

    pub fn is_closed(&self) -> bool {
        matches!(self, WsState::Closed | WsState::Closing)
    }
}

pub struct WsStream {
    ws: Arc<WebSysSocket>,
    queue: Arc<RefCell<VecDeque<Message>>>,
    stream_waker: Arc<RefCell<Option<Waker>>>,
    sink_waker: Arc<RefCell<Option<Waker>>>,
    _on_message: Arc<Closure<dyn FnMut(MessageEvent)>>,
    _on_close: Arc<Closure<dyn FnMut(JsCloseEvt)>>,
}

impl WsStream {
    pub(crate) fn new(ws: Arc<WebSysSocket>) -> Self {
        let queue = Arc::new(RefCell::new(VecDeque::new()));
        let stream_waker: Arc<RefCell<Option<Waker>>> = Arc::new(RefCell::new(None));
        let sink_waker: Arc<RefCell<Option<Waker>>> = Arc::new(RefCell::new(None));

        let q = Arc::clone(&queue);
        let w = Arc::clone(&stream_waker);
        let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Ok(msg) = convert_message(event) {
                q.borrow_mut().push_back(msg);
                if let Some(waker) = w.borrow_mut().take() {
                    waker.wake();
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        let w_stream = Arc::clone(&stream_waker);
        let w_sink = Arc::clone(&sink_waker);
        let on_close = Closure::wrap(Box::new(move |_: JsCloseEvt| {
            if let Some(waker) = w_stream.borrow_mut().take() {
                waker.wake();
            }
            if let Some(waker) = w_sink.borrow_mut().take() {
                waker.wake();
            }
        }) as Box<dyn FnMut(JsCloseEvt)>);

        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        Self {
            ws,
            queue,
            stream_waker,
            sink_waker,
            _on_message: Arc::new(on_message),
            _on_close: Arc::new(on_close),
        }
    }

    fn ready_state(&self) -> Result<WsState> {
        WsState::try_from(self.ws.ready_state())
    }
}

impl Stream for WsStream {
    type Item = Result<Message>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(msg) = self.queue.borrow_mut().pop_front() {
            return Poll::Ready(Some(Ok(msg)));
        }

        *self.stream_waker.borrow_mut() = Some(cx.waker().clone());

        match self.ready_state() {
            Ok(WsState::Closed) => Poll::Ready(None),
            Ok(WsState::Closing) => {
                if let Some(msg) = self.queue.borrow_mut().pop_front() {
                    Poll::Ready(Some(Ok(msg)))
                } else {
                    Poll::Pending
                }
            }
            Ok(_) => Poll::Pending,
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}

impl Sink<Message> for WsStream {
    type Error = Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        match self.ready_state() {
            Ok(WsState::Open) => Poll::Ready(Ok(())),
            Ok(WsState::Connecting) => {
                *self.sink_waker.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
            Ok(_) => Poll::Ready(Err(Error::NotConnected)),
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> std::result::Result<(), Self::Error> {
        match self.ready_state() {
            Ok(WsState::Open) => {
                match item {
                    Message::Binary(data) => {
                        self.ws
                            .send_with_u8_array(&data)
                            .map_err(|_| Error::NotConnected)?;
                    }
                    Message::Text(text) => {
                        self.ws
                            .send_with_str(&text)
                            .map_err(|_| Error::NotConnected)?;
                    }
                }
                Ok(())
            }
            Ok(_) => Err(Error::NotConnected),
            Err(e) => Err(e),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        match self.ready_state() {
            Ok(WsState::Open) | Ok(WsState::Connecting) => {
                let _ = self.ws.close();
                Poll::Ready(Ok(()))
            }
            _ => Poll::Ready(Ok(())),
        }
    }
}

impl Drop for WsStream {
    fn drop(&mut self) {
        self.ws.set_onmessage(None);
        self.ws.set_onclose(None);

        if self.ready_state().ok() == Some(WsState::Open) {
            let _ = self.ws.close();
        }
    }
}

fn convert_message(event: MessageEvent) -> Result<Message> {
    let data = event.data();

    if let Ok(array_buffer) = data.clone().dyn_into::<js_sys::ArrayBuffer>() {
        let array = js_sys::Uint8Array::new(&array_buffer);
        return Ok(Message::Binary(array.to_vec()));
    }

    if let Some(text) = data.as_string() {
        return Ok(Message::Text(text));
    }

    if data.is_instance_of::<web_sys::Blob>() {
        return Err(Error::Transport("blob data not supported".to_string()));
    }

    Err(Error::Transport("unknown message data type".to_string()))
}
