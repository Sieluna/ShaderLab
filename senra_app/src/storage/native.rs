use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use serde_json::Value;
use tokio::fs;

use super::{StorageError, StorageInner};

pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn new() -> Self {
        let exe_path = env::current_exe().unwrap();
        let path = exe_path.parent().unwrap().join("data.json");

        Self { path }
    }

    async fn load_data(&self) -> Result<HashMap<String, Value>, StorageError> {
        if self.path.exists() {
            let contents = fs::read(&self.path).await?;
            Ok(serde_json::from_slice(&contents)?)
        } else {
            Ok(HashMap::new())
        }
    }

    async fn save_data(&self, data: &HashMap<String, Value>) -> Result<(), StorageError> {
        let contents = serde_json::to_vec(data)?;
        fs::write(&self.path, contents).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInner for FileStorage {
    async fn save(&self, key: &str, value: Value) -> Result<(), StorageError> {
        let mut data = self.load_data().await?;
        data.insert(key.to_string(), value);
        self.save_data(&data).await
    }

    async fn load(&self, key: &str) -> Result<Option<Value>, StorageError> {
        let data = self.load_data().await?;
        Ok(data.get(key).cloned())
    }

    async fn remove(&self, key: &str) -> Result<(), StorageError> {
        let mut data = self.load_data().await?;
        if data.remove(key).is_some() {
            self.save_data(&data).await?;
        }
        Ok(())
    }
}
