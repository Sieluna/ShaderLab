use serde::{Deserialize, Serialize};

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
    pub created_at: String,
    pub updated_at: String,
}
