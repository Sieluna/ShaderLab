use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::resource::{CreateResourceRequest, ResourceResponse};
use crate::shader::{CreateShaderRequest, ShaderResponse};
use crate::user::UserPreviewResponse;

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotebookRequest {
    pub title: String,
    pub description: Option<String>,
    pub content: Value,
    pub resources: Vec<CreateResourceRequest>,
    pub shaders: Vec<CreateShaderRequest>,
    pub tags: Vec<String>,
    pub preview: Option<Vec<u8>>,
    pub visibility: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditNotebookRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<Value>,
    pub tags: Option<Vec<String>>,
    pub preview: Option<Vec<u8>>,
    pub visibility: Option<String>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotebookCommentRequest {
    pub content: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookStats {
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub is_liked: bool,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookInfo {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookPreviewResponse {
    #[serde(flatten)]
    pub inner: NotebookInfo,
    pub author: UserPreviewResponse,
    pub stats: NotebookStats,
    pub preview: Option<Vec<u8>>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookResponse {
    #[serde(flatten)]
    pub inner: NotebookInfo,
    pub author: UserPreviewResponse,
    pub stats: NotebookStats,
    pub content: Value,
    pub resources: Vec<ResourceResponse>,
    pub shaders: Vec<ShaderResponse>,
    pub visibility: String,
    pub version: i32,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookListResponse {
    pub notebooks: Vec<NotebookPreviewResponse>,
    pub total: i64,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookVersionResponse {
    pub id: i64,
    pub notebook_id: i64,
    pub version: i32,
    pub content: Value,
    pub created_at: String,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookVersionListResponse {
    pub versions: Vec<NotebookVersionResponse>,
    pub total: i64,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCommentItem {
    pub id: i64,
    pub notebook_id: i64,
    pub user_id: i64,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub author: String,
    pub author_avatar: Option<Vec<u8>>,
}

#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCommentListResponse {
    pub comments: Vec<NotebookCommentItem>,
    pub total: i64,
}
