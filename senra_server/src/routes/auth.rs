use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use senra_api::*;

use crate::errors::Result;
use crate::models::{CreateUser, LoginUser};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/verify", post(verify_token))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/auth/verify",
    tag = "auth",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Token verification successful", body = TokenResponse),
        (status = 401, description = "Invalid token")
    ),
    tag = "auth"
)]
async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<TokenResponse>> {
    let token = state.services.auth.refresh_token(&payload.token).await?;

    Ok(Json(TokenResponse { token }))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "auth"
)]
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let (user, token) = state
        .services
        .auth
        .login(LoginUser {
            username: payload.username,
            password: payload.password,
        })
        .await?;

    Ok(Json(AuthResponse {
        user: UserInfoResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar: user.avatar,
        },
        token,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "Username or email already exists")
    ),
    tag = "auth"
)]
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>> {
    let user = state
        .services
        .user
        .create_user(CreateUser {
            username: payload.username,
            email: payload.email,
            password: payload.password,
        })
        .await?;
    let token = state.services.auth.generate_token(user.id).await?;

    Ok(Json(AuthResponse {
        user: UserInfoResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar: user.avatar,
        },
        token,
    }))
}
