use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use url::Url;

use super::error::{Error, Result};
use super::{Message, WebSocket};

pub struct WsClient {
    socket: Option<WebSocket>,
    url: Option<Url>,
}

impl WsClient {
    pub fn new() -> Self {
        Self {
            socket: None,
            url: None,
        }
    }

    pub async fn connect(&mut self, url: Url) -> Result<()> {
        let socket = WebSocket::connect(&url, Duration::from_secs(10)).await?;
        self.socket = Some(socket);
        self.url = Some(url);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(mut socket) = self.socket.take() {
            socket.close().await?;
        }
        self.url = None;
        Ok(())
    }

    pub async fn send(&mut self, msg: Message) -> Result<()> {
        let socket = self.socket.as_mut().ok_or(Error::NotConnected)?;
        socket.send(msg).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<Message>> {
        let socket = self.socket.as_mut().ok_or(Error::NotConnected)?;
        match socket.next().await {
            Some(Ok(msg)) => Ok(Some(msg)),
            Some(Err(_)) => Err(Error::NotConnected),
            None => Ok(None),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }
}
