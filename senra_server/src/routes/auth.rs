use axum::extract::State;
use axum::routing::{patch, post};
use axum::{Json, Router};
use senra_api::*;

use crate::errors::Result;
use crate::middleware::AuthUser;
use crate::models::{CreateUser, EditUser, LoginUser};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/verify", post(verify_token))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/edit", patch(edit_user))
        .with_state(state)
}

async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>> {
    let auth_service = state.services.auth;
    let token = auth_service.refresh_token(&payload.token).await?;

    Ok(Json(VerifyResponse { token }))
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

    Ok(Json(AuthResponse {
        user: UserResponse {
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url.unwrap_or_default(),
        },
        token,
    }))
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

    Ok(Json(AuthResponse {
        user: UserResponse {
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url.unwrap_or_default(),
        },
        token,
    }))
}

async fn edit_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<EditRequest>,
) -> Result<Json<UserResponse>> {
    let user = state
        .services
        .auth
        .edit_user(
            auth_user.user_id,
            EditUser {
                username: payload.username,
                email: payload.email,
                password: payload.password,
                avatar_url: payload.avatar_url,
            },
        )
        .await?;

    Ok(Json(UserResponse {
        username: user.username,
        email: user.email,
        avatar_url: user.avatar_url.unwrap_or_default(),
    }))
}
