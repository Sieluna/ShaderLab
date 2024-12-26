use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notebook {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub content: Value,
    pub preview: Option<Vec<u8>>,
    pub visibility: String,
    pub version: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotebookVersion {
    pub id: i64,
    pub notebook_id: i64,
    pub user_id: i64,
    pub version: i32,
    pub content: Value,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotebookStats {
    pub notebook_id: i64,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotebookTag {
    pub id: i64,
    pub notebook_id: i64,
    pub tag: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotebookLike {
    pub id: i64,
    pub notebook_id: i64,
    pub user_id: i64,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotebookComment {
    pub id: i64,
    pub notebook_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNotebook {
    pub title: String,
    pub description: Option<String>,
    pub content: Value,
    pub tags: Vec<String>,
    pub preview: Option<Vec<u8>>,
    pub visibility: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNotebook {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<Value>,
    pub tags: Option<Vec<String>>,
    pub preview: Option<Vec<u8>>,
    pub visibility: Option<String>,
}
