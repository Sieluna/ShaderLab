use axum::extract::{Path, Query, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use senra_api::*;
use serde::Deserialize;

use crate::errors::Result;
use crate::middleware::AuthUser;
use crate::models::{CreateNotebook, CreateResource, CreateShader, UpdateNotebook};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
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

#[utoipa::path(
    get,
    path = "/notebooks",
    tag = "notebook",
    params(PaginationParams),
    responses(
        (status = 200, description = "Successfully retrieved notebook list", body = NotebookListResponse),
        (status = 401, description = "Unauthorized")
    )
)]
async fn list_notebooks(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<NotebookListResponse>> {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);

    let notebook_service = state.services.notebook;
    let (notebook_data, total) = notebook_service.list_notebooks(page, per_page).await?;

    let mut notebooks = Vec::new();
    for notebook in notebook_data {
        let stats = notebook_service.get_notebook_stats(notebook.id).await?;
        let tags = notebook_service.get_notebook_tags(notebook.id).await?;
        let is_liked = match auth_user.as_ref().map(|user| user.user_id) {
            Some(user_id) => {
                notebook_service
                    .is_notebook_liked(user_id, notebook.id)
                    .await?
            }
            None => false,
        };
        let user = state.services.user.get_user(notebook.user_id).await?;

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

    Ok(Json(NotebookListResponse { notebooks, total }))
}

#[utoipa::path(
    get,
    path = "/notebooks/{id}",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved notebook details", body = NotebookResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notebook not found")
    )
)]
async fn get_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<NotebookResponse>> {
    let notebook_service = state.services.notebook;
    let notebook = notebook_service.get_notebook(auth_user.user_id, id).await?;
    let stats = notebook_service.get_notebook_stats(id).await?;
    let tags = notebook_service.get_notebook_tags(id).await?;
    let is_liked = notebook_service
        .is_notebook_liked(auth_user.user_id, id)
        .await?;

    let resources = state.services.resource.get_resources(id).await?;
    let shaders = state.services.shader.get_shaders(id).await?;
    let user = state.services.user.get_user(notebook.user_id).await?;

    let resource_responses: Vec<ResourceResponse> = resources
        .into_iter()
        .map(|r| ResourceResponse {
            id: r.id,
            notebook_id: r.notebook_id,
            name: r.name,
            resource_type: r.resource_type,
            data: r.data,
            metadata: r.metadata,
            created_at: r.created_at.to_string(),
        })
        .collect();

    let shader_responses: Vec<ShaderResponse> = shaders
        .into_iter()
        .map(|s| ShaderResponse {
            id: s.id,
            notebook_id: s.notebook_id,
            name: s.name,
            shader_type: s.shader_type,
            code: s.code,
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        })
        .collect();

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
        resources: resource_responses,
        shaders: shader_responses,
        visibility: notebook.visibility,
        version: notebook.version,
    }))
}

#[utoipa::path(
    post,
    path = "/notebooks",
    tag = "notebook",
    request_body = CreateNotebookRequest,
    responses(
        (status = 200, description = "Successfully created notebook", body = NotebookResponse),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Invalid request data")
    )
)]
async fn create_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateNotebookRequest>,
) -> Result<Json<NotebookResponse>> {
    let resources: Vec<CreateResource> = payload
        .resources
        .into_iter()
        .map(|r| CreateResource {
            notebook_id: 0,
            name: r.name,
            resource_type: r.resource_type,
            data: r.data,
            metadata: r.metadata,
        })
        .collect();

    let shaders: Vec<CreateShader> = payload
        .shaders
        .into_iter()
        .map(|s| CreateShader {
            notebook_id: 0,
            name: s.name,
            shader_type: s.shader_type,
            code: s.code,
        })
        .collect();

    let notebook = state
        .services
        .notebook
        .create_notebook(
            auth_user.user_id,
            CreateNotebook {
                title: payload.title,
                description: payload.description,
                content: payload.content,
                resources,
                shaders,
                tags: payload.tags.clone(),
                preview: payload.preview,
                visibility: payload.visibility,
            },
        )
        .await?;

    let user = state.services.user.get_user(auth_user.user_id).await?;

    let notebook_service = state.services.notebook;
    let stats = notebook_service.get_notebook_stats(notebook.id).await?;
    let tags = notebook_service.get_notebook_tags(notebook.id).await?;

    let resources = state.services.resource.get_resources(notebook.id).await?;
    let shaders = state.services.shader.get_shaders(notebook.id).await?;

    let resource_responses: Vec<ResourceResponse> = resources
        .into_iter()
        .map(|r| ResourceResponse {
            id: r.id,
            notebook_id: r.notebook_id,
            name: r.name,
            resource_type: r.resource_type,
            data: r.data,
            metadata: r.metadata,
            created_at: r.created_at.to_string(),
        })
        .collect();

    let shader_responses: Vec<ShaderResponse> = shaders
        .into_iter()
        .map(|s| ShaderResponse {
            id: s.id,
            notebook_id: s.notebook_id,
            name: s.name,
            shader_type: s.shader_type,
            code: s.code,
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        })
        .collect();

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
        resources: resource_responses,
        shaders: shader_responses,
        visibility: notebook.visibility,
        version: notebook.version,
    }))
}

#[utoipa::path(
    patch,
    path = "/notebooks/{id}",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID")
    ),
    request_body = EditNotebookRequest,
    responses(
        (status = 200, description = "Successfully updated notebook", body = NotebookResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notebook not found")
    )
)]
async fn update_notebook(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<EditNotebookRequest>,
) -> Result<Json<NotebookResponse>> {
    let notebook_service = state.services.notebook;

    let notebook = notebook_service
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

    let user = state.services.user.get_user(auth_user.user_id).await?;

    let stats = notebook_service.get_notebook_stats(id).await?;
    let tags = notebook_service.get_notebook_tags(id).await?;
    let is_liked = notebook_service.is_notebook_liked(user.id, id).await?;

    let resources = state.services.resource.get_resources(id).await?;
    let shaders = state.services.shader.get_shaders(id).await?;

    let resource_responses: Vec<ResourceResponse> = resources
        .into_iter()
        .map(|r| ResourceResponse {
            id: r.id,
            notebook_id: r.notebook_id,
            name: r.name,
            resource_type: r.resource_type,
            data: r.data,
            metadata: r.metadata,
            created_at: r.created_at.to_string(),
        })
        .collect();

    let shader_responses: Vec<ShaderResponse> = shaders
        .into_iter()
        .map(|s| ShaderResponse {
            id: s.id,
            notebook_id: s.notebook_id,
            name: s.name,
            shader_type: s.shader_type,
            code: s.code,
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
        })
        .collect();

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
        resources: resource_responses,
        shaders: shader_responses,
        visibility: notebook.visibility,
        version: notebook.version,
    }))
}

#[utoipa::path(
    delete,
    path = "/notebooks/{id}",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted notebook"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notebook not found")
    )
)]
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

#[utoipa::path(
    get,
    path = "/notebooks/{id}/versions",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved notebook versions", body = NotebookVersionListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notebook not found")
    )
)]
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

#[utoipa::path(
    get,
    path = "/notebooks/{id}/comments",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved comments", body = NotebookCommentListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notebook not found")
    )
)]
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

#[utoipa::path(
    post,
    path = "/notebooks/{id}/comments",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID")
    ),
    request_body = CreateNotebookCommentRequest,
    responses(
        (status = 200, description = "Successfully created comment", body = NotebookCommentItem),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Notebook not found")
    )
)]
async fn create_comment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<CreateNotebookCommentRequest>,
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

#[utoipa::path(
    delete,
    path = "/notebooks/{id}/comments/{comment_id}",
    tag = "notebook",
    params(
        ("id" = i64, Path, description = "Notebook ID"),
        ("comment_id" = i64, Path, description = "Comment ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted comment"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Comment not found")
    )
)]
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
