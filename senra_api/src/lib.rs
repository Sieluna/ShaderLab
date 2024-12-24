mod auth;
mod notebook;

pub use auth::*;
use http::Method;
pub use notebook::*;
use serde::{Deserialize, Serialize};

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
                path: "/auth/edit",
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Token(TokenResponse),
    User(UserResponse),
    Auth(AuthResponse),

    Notebook(NotebookResponse),
    NotebookList(NotebookListResponse),
}
