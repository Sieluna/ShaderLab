use axum::{Json, Router};
use serde::Deserialize;

use crate::models::{CreateNotebook, Notebook, NotebookVersion, UpdateNotebook};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<i64>,
    per_page: Option<i64>,
}

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(list_notebooks).post(create_notebook))
        .route(
            "/:id",
            axum::routing::get(get_notebook)
                .patch(update_notebook)
                .delete(delete_notebook),
        )
        .route("/:id/versions", axum::routing::get(list_versions))
}

async fn list_notebooks(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let notebooks = sqlx::query_as!(
        Notebook,
        r#"
        SELECT * FROM notebooks
        ORDER BY updated_at DESC
        LIMIT $1 OFFSET $2
        "#,
        per_page,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok::<_, (StatusCode, String)>(Json(notebooks))
}

async fn create_notebook(
    State(state): State<AppState>,
    Json(payload): Json<CreateNotebook>,
) -> impl IntoResponse {
    let notebook = sqlx::query_as!(
        Notebook,
        r#"
        INSERT INTO notebooks (title, content)
        VALUES ($1, $2)
        RETURNING *
        "#,
        payload.title,
        payload.content
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok::<_, (StatusCode, String)>(Json(notebook))
}

async fn get_notebook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let notebook = sqlx::query_as!(
        Notebook,
        r#"
        SELECT * FROM notebooks
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Notebook not found".to_string()))?;

    Ok::<_, (StatusCode, String)>(Json(notebook))
}

async fn update_notebook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNotebook>,
) -> impl IntoResponse {
    let notebook = sqlx::query_as!(
        Notebook,
        r#"
        UPDATE notebooks
        SET
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            updated_at = CURRENT_TIMESTAMP,
            version = version + 1
        WHERE id = $3
        RETURNING *
        "#,
        payload.title,
        payload.content,
        id
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Notebook not found".to_string()))?;

    Ok::<_, (StatusCode, String)>(Json(notebook))
}

async fn delete_notebook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"
        DELETE FROM notebooks
        WHERE id = $1
        "#,
        id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok::<_, (StatusCode, String)>(StatusCode::NO_CONTENT)
}

async fn list_versions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let versions = sqlx::query_as!(
        NotebookVersion,
        r#"
        SELECT * FROM notebook_versions
        WHERE notebook_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        id,
        per_page,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok::<_, (StatusCode, String)>(Json(versions))
} 
 