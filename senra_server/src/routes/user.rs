use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use senra_api::*;
use serde::Deserialize;

use crate::errors::{Result, UserError};
use crate::middleware::AuthUser;
use crate::models::EditUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<i64>,
    per_page: Option<i64>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/user", get(get_user).patch(edit_user))
        .route("/user/{id}", get(get_user))
        .with_state(state)
}

async fn get_user(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    id: Option<Path<i64>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<UserResponse>> {
    let user_id = auth_user.map(|user| user.user_id);
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);
    let id = id
        .map(|id| id.0)
        .or(user_id)
        .ok_or(UserError::UserNotFound)?;

    let user = state.services.user.get_user(id).await?;

    let notebook_service = state.services.notebook;
    let (notebook_data, total) = notebook_service
        .list_notebooks_by_user(id, page, per_page)
        .await?;

    let mut notebooks = Vec::new();
    for notebook in notebook_data {
        let tags = notebook_service.get_notebook_tags(notebook.id).await?;
        let stats = notebook_service.get_notebook_stats(notebook.id).await?;
        let is_liked = match user_id {
            Some(user_id) => {
                notebook_service
                    .is_notebook_liked(user_id, notebook.id)
                    .await?
            }
            None => false,
        };

        notebooks.push(NotebookPreviewResponse {
            inner: NotebookInfo {
                id: notebook.id,
                title: notebook.title,
                description: notebook.description,
                tags: tags.into_iter().map(|tag| tag.tag).collect(),
                created_at: notebook.created_at.to_string(),
                updated_at: notebook.updated_at.to_string(),
            },
            author: UserPreviewResponse {
                id: user.id,
                username: user.username.clone(),
                avatar: Some(user.avatar.clone()),
            },
            stats: NotebookStats {
                view_count: stats.view_count,
                like_count: stats.like_count,
                comment_count: stats.comment_count,
                is_liked,
            },
            preview: notebook.preview,
        });
    }

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        avatar: Some(user.avatar),
        created_at: user.created_at.to_string(),
        notebooks: NotebookListResponse { notebooks, total },
    }))
}

async fn edit_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<EditUserRequest>,
) -> Result<Json<UserInfoResponse>> {
    let user = state
        .services
        .user
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

    Ok(Json(UserInfoResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        avatar: user.avatar,
    }))
}
