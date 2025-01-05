use axum::extract::{Path, Query, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use senra_api::*;
use serde::Deserialize;

use crate::errors::Result;
use crate::middleware::AuthUser;
use crate::models::{CreateNotebook, UpdateNotebook};
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
        .route(
            "/notebooks/{id}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/notebooks/{id}/comments/{comment_id}",
            delete(delete_comment),
        )
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

    let mut result = Vec::new();
    for notebook in notebooks {
        let stats = state
            .services
            .notebook
            .get_notebook_stats(notebook.id)
            .await?;
        let tags = state
            .services
            .notebook
            .get_notebook_tags(notebook.id)
            .await?;
        let is_liked = state
            .services
            .notebook
            .is_notebook_liked(auth_user.user_id, notebook.id)
            .await?;
        let user = state.services.user.get_user(notebook.user_id).await?;

        result.push(NotebookPreviewResponse {
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
                username: user.username,
                avatar: Some(user.avatar),
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

    Ok(Json(NotebookListResponse {
        notebooks: result,
        total,
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
    let stats = state.services.notebook.get_notebook_stats(id).await?;
    let tags = state.services.notebook.get_notebook_tags(id).await?;
    let is_liked = state
        .services
        .notebook
        .is_notebook_liked(auth_user.user_id, id)
        .await?;
    let user = state.services.user.get_user(notebook.user_id).await?;

    Ok(Json(NotebookResponse {
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
            username: user.username,
            avatar: Some(user.avatar),
        },
        stats: NotebookStats {
            view_count: stats.view_count,
            like_count: stats.like_count,
            comment_count: stats.comment_count,
            is_liked,
        },
        content: notebook.content,
        visibility: notebook.visibility,
        version: notebook.version,
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
        .create_notebook(
            auth_user.user_id,
            CreateNotebook {
                title: payload.title,
                description: payload.description,
                content: payload.content,
                tags: payload.tags.clone(),
                preview: payload.preview,
                visibility: payload.visibility,
            },
        )
        .await?;

    for tag in payload.tags {
        state
            .services
            .notebook
            .create_notebook_tag(notebook.id, tag)
            .await?;
    }

    state
        .services
        .notebook
        .create_notebook_stats(notebook.id)
        .await?;

    let stats = state
        .services
        .notebook
        .get_notebook_stats(notebook.id)
        .await?;
    let tags = state
        .services
        .notebook
        .get_notebook_tags(notebook.id)
        .await?;
    let user = state.services.user.get_user(auth_user.user_id).await?;

    Ok(Json(NotebookResponse {
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
            username: user.username,
            avatar: Some(user.avatar),
        },
        stats: NotebookStats {
            view_count: stats.view_count,
            like_count: stats.like_count,
            comment_count: stats.comment_count,
            is_liked: false,
        },
        content: notebook.content,
        visibility: notebook.visibility,
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
        .update_notebook(
            auth_user.user_id,
            id,
            UpdateNotebook {
                title: payload.title,
                description: payload.description,
                content: payload.content,
                tags: payload.tags,
                preview: payload.preview,
                visibility: payload.visibility,
            },
        )
        .await?;

    let stats = state.services.notebook.get_notebook_stats(id).await?;
    let tags = state.services.notebook.get_notebook_tags(id).await?;
    let is_liked = state
        .services
        .notebook
        .is_notebook_liked(auth_user.user_id, id)
        .await?;
    let user = state.services.user.get_user(auth_user.user_id).await?;

    Ok(Json(NotebookResponse {
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
            username: user.username,
            avatar: Some(user.avatar),
        },
        stats: NotebookStats {
            view_count: stats.view_count,
            like_count: stats.like_count,
            comment_count: stats.comment_count,
            is_liked,
        },
        content: notebook.content,
        visibility: notebook.visibility,
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
            })
            .collect(),
        total,
    }))
}

async fn list_comments(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<NotebookCommentListResponse>> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);

    let (comment_data, total) = state
        .services
        .notebook
        .list_comments(id, page, per_page)
        .await?;

    let mut comments = Vec::new();
    for comment in comment_data {
        let author = state.services.user.get_user(comment.user_id).await?;
        comments.push(NotebookCommentItem {
            id: comment.id,
            notebook_id: comment.notebook_id,
            user_id: comment.user_id,
            content: comment.content,
            created_at: comment.created_at.to_string(),
            updated_at: comment.updated_at.to_string(),
            author: author.username,
            author_avatar: Some(author.avatar),
        });
    }

    Ok(Json(NotebookCommentListResponse { comments, total }))
}

async fn create_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<Json<NotebookCommentItem>> {
    let comment = state
        .services
        .notebook
        .create_comment(auth_user.user_id, id, payload.content)
        .await?;

    let user = state.services.user.get_user(auth_user.user_id).await?;

    Ok(Json(NotebookCommentItem {
        id: comment.id,
        notebook_id: comment.notebook_id,
        user_id: comment.user_id,
        content: comment.content,
        created_at: comment.created_at.to_string(),
        updated_at: comment.updated_at.to_string(),
        author: user.username,
        author_avatar: Some(user.avatar),
    }))
}

async fn delete_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path((id, comment_id)): Path<(i64, i64)>,
) -> Result<()> {
    state
        .services
        .notebook
        .delete_comment(auth_user.user_id, id, comment_id)
        .await
}
