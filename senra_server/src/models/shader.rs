use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Shader {
    pub id: i64,
    pub notebook_id: i64,
    pub name: String,
    pub shader_type: String,
    pub code: String,
    pub version: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ShaderVersion {
    pub id: i64,
    pub shader_id: i64,
    pub version: i32,
    pub code: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateShader {
    pub notebook_id: i64,
    pub name: String,
    pub shader_type: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateShader {
    pub name: Option<String>,
    pub shader_type: Option<String>,
    pub code: Option<String>,
}
