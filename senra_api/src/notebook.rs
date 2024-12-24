use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotebookRequest {
    pub title: String,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotebookRequest {
    pub title: Option<String>,
    pub content: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookResponse {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: Value,
    pub created_at: String,
    pub updated_at: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookVersionResponse {
    pub id: i64,
    pub notebook_id: i64,
    pub version: i32,
    pub content: Value,
    pub created_at: String,
    pub created_by: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookListResponse {
    pub notebooks: Vec<NotebookResponse>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookVersionListResponse {
    pub versions: Vec<NotebookVersionResponse>,
    pub total: i64,
}
