use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<T> {
    pub id: String,
    pub message_type: String,
    pub payload: T,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse<T> {
    pub request_id: String,
    pub success: bool,
    pub payload: Option<T>,
    pub error: Option<String>,
}
