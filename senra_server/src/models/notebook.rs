use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notebook {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub content: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNotebook {
    pub title: String,
    pub content: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNotebook {
    pub title: Option<String>,
    pub content: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotebookVersion {
    pub id: i64,
    pub notebook_id: i64,
    pub version: i32,
    pub content: Value,
    pub created_at: OffsetDateTime,
    pub created_by: i64,
}
