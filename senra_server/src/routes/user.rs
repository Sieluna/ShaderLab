use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use senra_api::*;

use crate::errors::Result;
use crate::middleware::AuthUser;
use crate::models::EditUser;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/user", get(get_user).patch(edit_user))
        .with_state(state)
}

async fn get_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>> {
    let user_service = state.services.user;
    let user = user_service.get_user(auth_user.user_id).await?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        avatar: user.avatar,
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
