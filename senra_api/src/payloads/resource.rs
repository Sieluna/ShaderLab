use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
}
