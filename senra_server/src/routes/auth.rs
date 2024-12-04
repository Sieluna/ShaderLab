use axum::extract::State;
use axum::routing::{patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::middleware::AuthUser;
use crate::models::{CreateUser, EditUser, LoginUser, User};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/edit", patch(edit_user))
        .with_state(state)
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let auth_service = state.services.auth;
    let (user, token) = auth_service
        .login(LoginUser {
            username: payload.username,
            password: payload.password,
        })
        .await?;

    Ok(Json(AuthResponse { user, token }))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    let auth_service = state.services.auth;
    let (user, token) = auth_service
        .register(CreateUser {
            username: payload.username,
            email: payload.email,
            password: payload.password,
        })
        .await?;

    Ok(Json(AuthResponse { user, token }))
}

async fn edit_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<EditRequest>,
) -> Result<Json<User>> {
    let user = state
        .services
        .auth
        .edit_user(auth_user.user_id, EditUser {
            username: payload.username,
            email: payload.email,
            password: payload.password,
            avatar_url: payload.avatar_url,
        })
        .await?;

    Ok(Json(user))
}
