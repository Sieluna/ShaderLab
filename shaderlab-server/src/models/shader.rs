use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Shader {
    pub id: i64,
    pub notebook_id: i64,
    pub code: String,
    pub position: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShaderDTO {
    pub notebook_id: i64,
    pub code: String,
    pub position: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileShaderDTO {
    pub code: String,
}
