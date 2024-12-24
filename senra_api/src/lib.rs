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

pub const AUTH_TOKEN: Endpoint = Endpoint {
    path: "/auth/verify",
    method: Method::POST,
};
pub const LOGIN: Endpoint = Endpoint {
    path: "/auth/login",
    method: Method::POST,
};
pub const REGISTER: Endpoint = Endpoint {
    path: "/auth/register",
    method: Method::POST,
};
pub const EDIT_USER: Endpoint = Endpoint {
    path: "/auth/edit",
    method: Method::PATCH,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Verify(VerifyRequest),
    Login(LoginRequest),
    Register(RegisterRequest),
    EditUser(EditRequest),
}

impl From<Request> for Endpoint {
    fn from(request: Request) -> Self {
        match request {
            Request::Verify(_) => AUTH_TOKEN,
            Request::Login(_) => LOGIN,
            Request::Register(_) => REGISTER,
            Request::EditUser(_) => EDIT_USER,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Verify(VerifyResponse),
    User(UserResponse),
    Auth(AuthResponse),
}
