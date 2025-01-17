use std::io::{Error, ErrorKind};

use serde_json::Value;
use wasm_bindgen::prelude::*;
use web_sys::{Storage, window};

use super::{StorageError, StorageInner};

use crate::config::Config;

impl From<JsValue> for StorageError {
    fn from(error: JsValue) -> Self {
        let error = Error::new(ErrorKind::Other, format!("{:?}", error));
        StorageError::Io(error)
    }
}

pub struct WebStorage {
    storage: Storage,
}

unsafe impl Send for WebStorage {}
unsafe impl Sync for WebStorage {}

impl WebStorage {
    pub fn new(config: &Config) -> Self {
        let storage = window()
            .unwrap()
            .local_storage()
            .unwrap()
            .expect("localStorage should be available");

        Self { storage }
    }
}

#[async_trait::async_trait]
impl StorageInner for WebStorage {
    async fn save(&self, key: &str, value: Value) -> Result<(), StorageError> {
        let value_str = serde_json::to_string(&value)?;
        self.storage.set_item(key, &value_str)?;
        Ok(())
    }

    async fn load(&self, key: &str) -> Result<Option<Value>, StorageError> {
        let value_str = self.storage.get_item(key)?;

        match value_str {
            Some(str) => Ok(Some(serde_json::from_str(&str)?)),
            None => Ok(None),
        }
    }

    async fn remove(&self, key: &str) -> Result<(), StorageError> {
        self.storage.remove_item(key)?;
        Ok(())
    }
}
