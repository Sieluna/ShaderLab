use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub url: String,
    pub storage_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: option_env!("URL")
                .unwrap_or("http://localhost:3000")
                .to_string(),
            storage_path: option_env!("STORAGE_PATH")
                .unwrap_or("./data.json")
                .to_string(),
        }
    }
}
