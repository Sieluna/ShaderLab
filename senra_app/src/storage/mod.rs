#[cfg(not(target_arch = "wasm32"))]
mod native;

use iced::Task;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Path error: {0}")]
    PathError(String),
}

#[derive(Debug, Clone)]
pub enum StorageMessage {
    Save(String, Value),
    Load(String),
    Saved(String, bool),
    Loaded(String, Option<Value>),
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
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let storage = native::FileStorage::new();
            Self {
                inner: Arc::new(storage),
            }
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

    pub fn update(&mut self, message: StorageMessage) -> Task<StorageMessage> {
        match message {
            StorageMessage::Save(key, value) => {
                let inner = self.inner.clone();
                Task::perform(
                    async move {
                        inner.save(&key, value).await.map_err(|e| e.to_string())?;
                        Ok(key)
                    },
                    |result| match result {
                        Ok(key) => StorageMessage::Saved(key, true),
                        Err(e) => StorageMessage::Error(e),
                    },
                )
            }
            StorageMessage::Load(key) => {
                let inner = self.inner.clone();
                Task::perform(
                    async move {
                        let result = inner.load(&key).await.map_err(|e| e.to_string())?;
                        Ok((key, result))
                    },
                    |result| match result {
                        Ok((key, value)) => StorageMessage::Loaded(key, value),
                        Err(e) => StorageMessage::Error(e),
                    },
                )
            }
            _ => Task::none(),
        }
    }
}
