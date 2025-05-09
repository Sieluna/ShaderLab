#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

use std::sync::Arc;

use iced::Task;
use serde_json::Value;

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub enum Message {
    GetRequest(String),
    SetRequest(String, Value),
    RemoveRequest(String),

    GetRespond(String, Option<Value>),
    SetRespond(String),
    RemoveRespond(String),

    Error(String),
}

#[async_trait::async_trait]
pub trait StorageInner: Send + Sync {
    async fn save(&self, key: &str, value: Value) -> Result<(), StorageError>;

    async fn load(&self, key: &str) -> Result<Option<Value>, StorageError>;

    async fn remove(&self, key: &str) -> Result<(), StorageError>;
}

#[derive(Clone)]
pub struct Storage {
    inner: Arc<dyn StorageInner>,
}

impl Storage {
    pub fn new(config: &Config) -> Self {
        let storage = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                native::FileStorage::new(config)
            }
            #[cfg(target_arch = "wasm32")]
            {
                web::WebStorage::new(config)
            }
        };

        Self {
            inner: Arc::new(storage),
        }
    }

    pub async fn save(&self, key: &str, value: Value) -> bool {
        self.inner.save(key, value).await.is_ok()
    }

    pub async fn load(&self, key: &str) -> Option<Value> {
        self.inner.load(key).await.ok()?
    }

    pub async fn remove(&self, key: &str) -> bool {
        self.inner.remove(key).await.is_ok()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetRequest(key, value) => {
                let inner = self.inner.clone();
                Task::perform(
                    async move {
                        inner.save(&key, value).await.map_err(|e| e.to_string())?;
                        Ok(key)
                    },
                    |result| match result {
                        Ok(key) => Message::SetRespond(key),
                        Err(e) => Message::Error(e),
                    },
                )
            }
            Message::GetRequest(key) => {
                let inner = self.inner.clone();
                Task::perform(
                    async move {
                        let result = inner.load(&key).await.map_err(|e| e.to_string())?;
                        Ok((key, result))
                    },
                    |result| match result {
                        Ok((key, value)) => Message::GetRespond(key, value),
                        Err(e) => Message::Error(e),
                    },
                )
            }
            Message::RemoveRequest(key) => {
                let inner = self.inner.clone();
                Task::perform(
                    async move {
                        inner.remove(&key).await.map_err(|e| e.to_string())?;
                        Ok(key)
                    },
                    |result| match result {
                        Ok(key) => Message::RemoveRespond(key),
                        Err(e) => Message::Error(e),
                    },
                )
            }
            _ => Task::none(),
        }
    }
}
