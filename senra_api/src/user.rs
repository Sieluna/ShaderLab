use serde::{Deserialize, Serialize};

use crate::notebook::NotebookListResponse;

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub avatar: Option<Vec<u8>>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreviewResponse {
    pub id: i64,
    pub username: String,
    pub avatar: Option<Vec<u8>>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub avatar: Vec<u8>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub avatar: Option<Vec<u8>>,
    pub created_at: String,
    pub notebooks: NotebookListResponse,
}
