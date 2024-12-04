use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i64,
    pub notebook_id: i64,
    pub name: String,
    pub resource_type: String,
    pub data: Vec<u8>,
    pub metadata: Option<Value>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResource {
    pub notebook_id: i64,
    pub name: String,
    pub resource_type: String,
    pub data: Vec<u8>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResource {
    pub name: Option<String>,
    pub data: Option<Vec<u8>>,
    pub metadata: Option<Value>,
}
