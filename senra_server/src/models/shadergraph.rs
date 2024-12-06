use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ShaderGraph {
    pub id: i64,
    pub notebook_id: i64,
    pub name: String,
    pub graph_data: Value,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShaderGraph {
    pub notebook_id: i64,
    pub name: String,
    pub graph_data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateShaderGraph {
    pub name: Option<String>,
    pub graph_data: Option<Value>,
}
