use core::ops::DerefMut;
use core::pin::Pin;
use core::result::Result;
use core::task::{Context, Poll};
use core::time::Duration;

use futures_util::{Sink, Stream};
use url::Url;

use crate::ws::{Error, Message};

#[cfg(not(target_arch = "wasm32"))]
use tokio::net::TcpStream;

#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[cfg(target_arch = "wasm32")]
use crate::ws::wasm::WsStream;

#[cfg(not(target_arch = "wasm32"))]
type WsStream<T> = WebSocketStream<MaybeTlsStream<T>>;

pub enum WebSocket {
    #[cfg(not(target_arch = "wasm32"))]
    Tokio(Box<WsStream<TcpStream>>),
    #[cfg(target_arch = "wasm32")]
    Wasm(WsStream),
}

impl WebSocket {
    pub(crate) async fn connect(url: &Url, timeout: Duration) -> Result<Self, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use tokio::time;
            use tokio_tungstenite::connect_async;

            let (stream, _) = time::timeout(timeout, connect_async(url.as_str()))
                .await
                .map_err(|_| Error::Timeout)?
                .map_err(Error::Transport)?;
            Ok(WebSocket::Tokio(Box::new(stream)))
        }

        #[cfg(target_arch = "wasm32")]
        {
            let (_ws, stream) = crate::ws::wasm::connect(url)
                .await
                .map_err(|e| Error::Transport(e.to_string()))?;
            Ok(WebSocket::Wasm(stream))
        }
    }
}

impl Sink<Message> for WebSocket {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s.as_mut())
                .poll_ready(cx)
                .map_err(Error::Transport),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_ready(cx),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s.as_mut())
                .start_send(item.into())
                .map_err(Error::Transport),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).start_send(item),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s.as_mut())
                .poll_flush(cx)
                .map_err(Error::Transport),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s.as_mut())
                .poll_close(cx)
                .map_err(Error::Transport),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_close(cx),
        }
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s)
                .poll_next(cx)
                .map(|opt| opt.map(|res| res.map(|msg| msg.into()).map_err(Error::Transport))),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_next(cx),
        }
    }
}
