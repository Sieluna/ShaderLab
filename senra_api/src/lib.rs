mod auth;
mod notebook;
mod user;

pub use auth::*;
use http::Method;
pub use notebook::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub use user::*;

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub path: String,
    pub method: Method,
    pub body: Option<Value>,
    pub params: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
}

impl Endpoint {
    pub fn new(path: &'static str) -> Self {
        Self {
            path: path.to_string(),
            method: Method::GET,
            body: None,
            params: Vec::new(),
            query: Vec::new(),
        }
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn with_param(mut self, key: &str, value: impl ToString) -> Self {
        self.params.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_query(mut self, key: &str, value: impl ToString) -> Self {
        self.query.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_body<T: Serialize>(mut self, body: T) -> Result<Self, serde_json::Error> {
        self.body = Some(serde_json::to_value(body)?);
        Ok(self)
    }

    pub fn build_url(&self) -> String {
        let mut path = self.path.clone();

        for (key, value) in &self.params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        if !self.query.is_empty() {
            let query_string = self
                .query
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            path = format!("{}?{}", path, query_string);
        }

        path
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Auth(AuthRequest),
    Login(LoginRequest),
    Register(RegisterRequest),
    GetUser(u64),
    EditUser(EditUserRequest),

    GetNotebookList {
        page: Option<u32>,
        limit: Option<u32>,
        category: Option<String>,
        search: Option<String>,
    },
    GetNotebook(u64),
    CreateNotebook(CreateNotebookRequest),
    EditNotebook(u64, EditNotebookRequest),
    RemoveNotebook(u64),

    LikeNotebook(u64),
    UnlikeNotebook(u64),

    GetCommentList {
        page: Option<u32>,
        limit: Option<u32>,
    },
    CreateComment(u64, String),
}

impl TryFrom<Request> for Endpoint {
    type Error = serde_json::Error;

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
            Request::GetUser(id) => Endpoint::new("/user/{id}").with_param("id", id),
            Request::EditUser(req) => Endpoint::new("/user")
                .with_method(Method::PATCH)
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
            Request::CreateNotebook(req) => Endpoint::new("/notebooks")
                .with_method(Method::POST)
                .with_body(req)?,
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
            Request::CreateComment(id, content) => Endpoint::new("/notebooks/{id}/comments")
                .with_method(Method::POST)
                .with_body(json!({ "comment": content }))?
                .with_param("id", id),
        })
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

impl Response {
    pub fn from_body(
        endpoint: &Endpoint,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        match endpoint.path.as_str() {
            "/auth/verify" => Ok(Response::Auth(serde_json::from_value(value)?)),
            "/auth/login" => Ok(Response::Token(serde_json::from_value(value)?)),
            "/auth/register" => Ok(Response::User(serde_json::from_value(value)?)),
            "/user" | "/user/{id}" => Ok(Response::User(serde_json::from_value(value)?)),
            "/notebooks" => Ok(Response::NotebookList(serde_json::from_value(value)?)),
            "/notebooks/{id}" => Ok(Response::Notebook(serde_json::from_value(value)?)),
            "/notebooks/{id}/comments" => {
                Ok(Response::NotebookList(serde_json::from_value(value)?))
            }
            _ => serde_json::from_value(value),
        }
    }
}
