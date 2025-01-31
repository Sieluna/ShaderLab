use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShaderRequest {
    pub notebook_id: i64,
    pub name: String,
    pub shader_type: String,
    pub code: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditShaderRequest {
    pub name: Option<String>,
    pub shader_type: Option<String>,
    pub code: Option<String>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderResponse {
    pub id: i64,
    pub notebook_id: i64,
    pub name: String,
    pub shader_type: String,
    pub code: String,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderVersionResponse {
    pub id: i64,
    pub shader_id: i64,
    pub version: i32,
    pub code: String,
    pub created_at: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderVersionListResponse {
    pub versions: Vec<ShaderVersionResponse>,
    pub total: i64,
}
