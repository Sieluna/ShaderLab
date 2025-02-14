mod auth;
mod client;
#[cfg(target_arch = "wasm32")]
mod client_wasm;
mod endpoint;
mod notebook;
mod resource;
mod shader;
mod user;

use http::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use auth::*;
pub use client::*;
#[cfg(target_arch = "wasm32")]
pub use client_wasm::*;
pub use endpoint::*;
pub use notebook::*;
pub use resource::*;
pub use shader::*;
pub use user::*;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::NetworkError(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Auth(AuthRequest),
    Login(LoginRequest),
    Register(RegisterRequest),
    GetSelf,
    GetUser(u64),
    EditUser(EditUserRequest),

    CreateNotebook(CreateNotebookRequest),
    GetNotebookList {
        page: Option<u32>,
        limit: Option<u32>,
        category: Option<String>,
        search: Option<String>,
    },
    GetNotebook(u64),
    EditNotebook(u64, EditNotebookRequest),
    RemoveNotebook(u64),

    UpdateShader {
        notebook_id: i64,
        shader_id: i64,
        code: String,
    },
    UpdateResource {
        notebook_id: i64,
        resource_id: i64,
        data: Vec<u8>,
        metadata: Option<serde_json::Value>,
    },

    LikeNotebook(u64),
    UnlikeNotebook(u64),

    CreateComment(u64, String),
    GetCommentList {
        page: Option<u32>,
        limit: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Token(TokenResponse),
    User(UserResponse),
    Auth(AuthResponse),

    Notebook(NotebookResponse),
    NotebookList(NotebookListResponse),

    Comment(NotebookCommentResponse),
    CommentList(NotebookCommentListResponse),
}

impl TryFrom<Request> for Endpoint {
    type Error = ApiError;

    fn try_from(request: Request) -> Result<Self, Self::Error> {
        Ok(match request {
            Request::Auth(req) => Endpoint::new("/auth/verify")
                .with_method(Method::POST)
                .with_body(req)?,
            Request::Login(req) => Endpoint::new("/auth/login")
                .with_method(Method::POST)
                .with_body(req)?,
            Request::Register(req) => Endpoint::new("/auth/register")
                .with_method(Method::POST)
                .with_body(req)?,
            Request::GetSelf => Endpoint::new("/user"),
            Request::GetUser(id) => Endpoint::new("/user/{id}").with_param("id", id),
            Request::EditUser(req) => Endpoint::new("/user")
                .with_method(Method::PATCH)
                .with_body(req)?,

            Request::CreateNotebook(req) => Endpoint::new("/notebooks")
                .with_method(Method::POST)
                .with_body(req)?,
            Request::GetNotebookList {
                page,
                limit,
                category,
                search,
            } => {
                let mut endpoint = Endpoint::new("/notebooks");
                if let Some(page) = page {
                    endpoint = endpoint.with_query("page", page);
                }
                if let Some(limit) = limit {
                    endpoint = endpoint.with_query("limit", limit);
                }
                if let Some(category) = category {
                    endpoint = endpoint.with_query("category", category);
                }
                if let Some(search) = search {
                    endpoint = endpoint.with_query("search", search);
                }
                endpoint
            }
            Request::GetNotebook(id) => Endpoint::new("/notebooks/{id}").with_param("id", id),
            Request::EditNotebook(id, req) => Endpoint::new("/notebooks/{id}")
                .with_method(Method::PATCH)
                .with_body(req)?
                .with_param("id", id),
            Request::RemoveNotebook(id) => Endpoint::new("/notebooks/{id}")
                .with_method(Method::DELETE)
                .with_param("id", id),

            Request::LikeNotebook(id) => Endpoint::new("/notebooks/{id}/like")
                .with_method(Method::POST)
                .with_param("id", id),
            Request::UnlikeNotebook(id) => Endpoint::new("/notebooks/{id}/unlike")
                .with_method(Method::POST)
                .with_param("id", id),

            Request::CreateComment(id, content) => Endpoint::new("/notebooks/{id}/comments")
                .with_method(Method::POST)
                .with_body(json!({ "comment": content }))?
                .with_param("id", id),
            Request::GetCommentList { page, limit } => {
                let mut endpoint = Endpoint::new("/notebooks/{id}/comments");
                if let Some(page) = page {
                    endpoint = endpoint.with_query("page", page);
                }
                if let Some(limit) = limit {
                    endpoint = endpoint.with_query("limit", limit);
                }
                endpoint
            }

            _ => Err(ApiError::UnknownError("Invalid Http Endpoint".to_string()))?,
        })
    }
}
