use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceRequest {
    pub notebook_id: i64,
    pub name: String,
    pub resource_type: String,
    pub data: Vec<u8>,
    pub metadata: Option<Value>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditResourceRequest {
    pub name: Option<String>,
    pub data: Option<Vec<u8>>,
    pub metadata: Option<Value>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub id: i64,
    pub notebook_id: i64,
    pub name: String,
    pub resource_type: String,
    pub data: Vec<u8>,
    pub metadata: Option<Value>,
    pub created_at: String,
}
