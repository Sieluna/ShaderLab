mod client;
mod error;
pub mod message;
pub mod socket;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use client::WsClient;
pub use error::{Error, Result};
pub use message::Message;
pub use socket::WebSocket;
