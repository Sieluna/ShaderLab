mod auth;

pub use auth::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Login(LoginRequest),
    Register(RegisterRequest),
    EditUser(EditRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    User(UserResponse),
    Auth(AuthResponse),
}
