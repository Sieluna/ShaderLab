use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use senra_api::*;
use serde::Deserialize;

use crate::errors::Result;
use crate::middleware::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<i64>,
    per_page: Option<i64>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/notebooks", get(list_notebooks).post(create_notebook))
        .route(
            "/notebooks/{id}",
            get(get_notebook)
                .patch(update_notebook)
                .delete(delete_notebook),
        )
        .route("/notebooks/{id}/versions", get(list_versions))
        .with_state(state)
}

async fn list_notebooks(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<NotebookListResponse>> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);

    let (notebooks, total) = state
        .services
        .notebook
        .list_notebooks(auth_user.user_id, page, per_page)
        .await?;

    Ok(Json(NotebookListResponse {
        notebooks: notebooks
            .into_iter()
            .map(|n| NotebookResponse {
                id: n.id,
                user_id: n.user_id,
                title: n.title,
                content: n.content,
                created_at: n.created_at.to_string(),
                updated_at: n.updated_at.to_string(),
                version: n.version,
            })
            .collect(),
        total,
    }))
}

async fn create_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateNotebookRequest>,
) -> Result<Json<NotebookResponse>> {
    let notebook = state
        .services
        .notebook
        .create_notebook(auth_user.user_id, payload.title, payload.content)
        .await?;

    Ok(Json(NotebookResponse {
        id: notebook.id,
        user_id: notebook.user_id,
        title: notebook.title,
        content: notebook.content,
        created_at: notebook.created_at.to_string(),
        updated_at: notebook.updated_at.to_string(),
        version: notebook.version,
    }))
}

async fn get_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<NotebookResponse>> {
    let notebook = state
        .services
        .notebook
        .get_notebook(auth_user.user_id, id)
        .await?;

    Ok(Json(NotebookResponse {
        id: notebook.id,
        user_id: notebook.user_id,
        title: notebook.title,
        content: notebook.content,
        created_at: notebook.created_at.to_string(),
        updated_at: notebook.updated_at.to_string(),
        version: notebook.version,
    }))
}

async fn update_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<EditNotebookRequest>,
) -> Result<Json<NotebookResponse>> {
    let notebook = state
        .services
        .notebook
        .update_notebook(auth_user.user_id, id, payload.title, payload.content)
        .await?;

    Ok(Json(NotebookResponse {
        id: notebook.id,
        user_id: notebook.user_id,
        title: notebook.title,
        content: notebook.content,
        created_at: notebook.created_at.to_string(),
        updated_at: notebook.updated_at.to_string(),
        version: notebook.version,
    }))
}

async fn delete_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
) -> Result<()> {
    state
        .services
        .notebook
        .delete_notebook(auth_user.user_id, id)
        .await
}

async fn list_versions(
    State(state): State<AppState>,
    _auth_user: AuthUser,
    Path(id): Path<i64>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<NotebookVersionListResponse>> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);

    let (versions, total) = state
        .services
        .notebook
        .list_versions(id, page, per_page)
        .await?;

    Ok(Json(NotebookVersionListResponse {
        versions: versions
            .into_iter()
            .map(|v| NotebookVersionResponse {
                id: v.id,
                notebook_id: v.notebook_id,
                version: v.version,
                content: v.content,
                created_at: v.created_at.to_string(),
                created_by: v.created_by,
            })
            .collect(),
        total,
    }))
}
