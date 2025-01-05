mod auth;
mod notebook;
mod user;

pub use auth::*;
use http::Method;
pub use notebook::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
pub use user::*;

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub path: &'static str,
    pub method: Method,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Auth(AuthRequest),
    Login(LoginRequest),
    Register(RegisterRequest),
    EditUser(EditUserRequest),

    GetNotebookList,
    GetNotebook(u64),
    CreateNotebook(CreateNotebookRequest),
    EditNotebook(u64, EditNotebookRequest),
    RemoveNotebook(u64),

    LikeNotebook(u64),
    UnlikeNotebook(u64),

    GetCommentList,
    CreateComment(u64, String),
}

impl Request {
    pub fn serialize_for_http(&self) -> serde_json::Value {
        match self {
            Request::Auth(req) => serde_json::to_value(req).unwrap(),
            Request::Login(req) => serde_json::to_value(req).unwrap(),
            Request::Register(req) => serde_json::to_value(req).unwrap(),
            Request::EditUser(req) => serde_json::to_value(req).unwrap(),

            Request::GetNotebookList => serde_json::Value::Null,
            Request::GetNotebook(id) => serde_json::to_value(id).unwrap(),
            Request::CreateNotebook(req) => serde_json::to_value(req).unwrap(),
            Request::EditNotebook(_, req) => serde_json::to_value(req).unwrap(),
            Request::RemoveNotebook(id) => serde_json::to_value(id).unwrap(),

            Request::LikeNotebook(id) => serde_json::to_value(id).unwrap(),
            Request::UnlikeNotebook(id) => serde_json::to_value(id).unwrap(),

            Request::GetCommentList => serde_json::Value::Null,
            Request::CreateComment(id, content) => {
                json!({
                    "notebook_id": id,
                    "comment": content,
                })
            }
        }
    }
}

impl From<Request> for Endpoint {
    fn from(request: Request) -> Self {
        match request {
            Request::Auth(_) => Endpoint {
                path: "/auth/verify",
                method: Method::POST,
            },
            Request::Login(_) => Endpoint {
                path: "/auth/login",
                method: Method::POST,
            },
            Request::Register(_) => Endpoint {
                path: "/auth/register",
                method: Method::POST,
            },
            Request::EditUser(_) => Endpoint {
                path: "/user",
                method: Method::PATCH,
            },
            Request::GetNotebookList => Endpoint {
                path: "/notebooks",
                method: Method::GET,
            },
            Request::GetNotebook(_) => Endpoint {
                path: "/notebooks/{id}",
                method: Method::GET,
            },
            Request::CreateNotebook(_) => Endpoint {
                path: "/notebooks",
                method: Method::POST,
            },
            Request::EditNotebook(_, _) => Endpoint {
                path: "/notebooks/{id}",
                method: Method::PATCH,
            },
            Request::RemoveNotebook(_) => Endpoint {
                path: "/notebooks/{id}",
                method: Method::DELETE,
            },
            Request::LikeNotebook(_) => Endpoint {
                path: "/notebooks/{id}/like",
                method: Method::POST,
            },
            Request::UnlikeNotebook(_) => Endpoint {
                path: "/notebooks/{id}/unlike",
                method: Method::POST,
            },
            Request::GetCommentList => Endpoint {
                path: "/notebooks/{id}/comments",
                method: Method::GET,
            },
            Request::CreateComment(_, _) => Endpoint {
                path: "/notebooks/{id}/comments",
                method: Method::POST,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Token(TokenResponse),
    User(UserInfoResponse),
    Auth(AuthResponse),

    Notebook(NotebookResponse),
    NotebookList(NotebookListResponse),
}

impl Response {
    pub fn deserialize_from_http(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if let Ok(auth) = serde_json::from_value::<AuthResponse>(value.clone()) {
            return Ok(Response::Auth(auth));
        }
        if let Ok(token) = serde_json::from_value::<TokenResponse>(value.clone()) {
            return Ok(Response::Token(token));
        }

        if let Ok(notebook) = serde_json::from_value::<NotebookResponse>(value.clone()) {
            return Ok(Response::Notebook(notebook));
        }
        if let Ok(notebook_list) = serde_json::from_value::<NotebookListResponse>(value.clone()) {
            return Ok(Response::NotebookList(notebook_list));
        }

        serde_json::from_value(value)
    }
}
