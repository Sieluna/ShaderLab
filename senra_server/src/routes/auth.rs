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
    Json(payload): Json<AuthRequest>,
) -> Result<Json<TokenResponse>> {
    let auth_service = state.services.auth;
    let token = auth_service.refresh_token(&payload.token).await?;

    Ok(Json(TokenResponse { token }))
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
            id: user.id,
            username: user.username,
            email: user.email,
            avatar: user.avatar,
        },
        token,
    }))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    let user_service = state.services.user;
    let user = user_service
        .create_user(CreateUser {
            username: payload.username,
            email: payload.email,
            password: payload.password,
        })
        .await?;
    let auth_service = state.services.auth;
    let token = auth_service.generate_token(user.id).await?;

    Ok(Json(AuthResponse {
        user: UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar: user.avatar,
        },
        token,
    }))
}

async fn edit_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<EditUserRequest>,
) -> Result<Json<UserResponse>> {
    let user_service = state.services.user;
    let user = user_service
        .edit_user(
            auth_user.user_id,
            EditUser {
                username: payload.username,
                email: payload.email,
                password: payload.password,
                avatar: payload.avatar,
            },
        )
        .await?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        avatar: user.avatar,
    }))
}
