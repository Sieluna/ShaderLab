mod auth;
mod notebook;
mod user;
mod ws;

use axum::Router;
use axum::http::Method;
use axum::response::{Html, Json};
use axum::routing::get;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(auth::router(state.clone()))
        .merge(notebook::router(state.clone()))
        .merge(user::router(state.clone()))
        .merge(ws::router(state.clone()))
        .merge(openapi())
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_headers(Any)
                .allow_origin(Any),
        )
}

fn openapi() -> Router {
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(
        paths(
            auth::verify_token,
            auth::login,
            auth::register,
            user::get_self,
            user::get_user,
            user::edit_user,
            notebook::list_notebooks,
            notebook::get_notebook,
            notebook::create_notebook,
            notebook::update_notebook,
            notebook::delete_notebook,
            notebook::list_versions,
            notebook::list_comments,
            notebook::create_comment,
            notebook::delete_comment
        ),
        components(
            schemas(
                senra_api::AuthRequest,
                senra_api::AuthResponse,
                senra_api::LoginRequest,
                senra_api::RegisterRequest,
                senra_api::UserResponse,
                senra_api::UserInfoResponse,
                senra_api::EditUserRequest,
                senra_api::NotebookListResponse,
                senra_api::NotebookResponse,
                senra_api::CreateNotebookRequest,
                senra_api::EditNotebookRequest,
                senra_api::NotebookVersionListResponse,
                senra_api::NotebookCommentListResponse,
                senra_api::CreateNotebookCommentRequest,
                senra_api::NotebookCommentResponse
            )
        ),
        tags(
            (name = "auth", description = "Authentication related endpoints"),
            (name = "user", description = "User related endpoints"),
            (name = "notebook", description = "Notebook related endpoints")
        )
    )]
    struct ApiDoc;

    const OPENAPI_ENDPOINT: &str = "/openapi.json";

    Router::new()
        .route(OPENAPI_ENDPOINT, get(||async { Json(ApiDoc::openapi()) }))
        .route("/", get(|| async {
            Html(format!(
                r#"
                <!doctype html>
                <html>
                <head>
                    <meta charset="utf-8">
                    <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
                </head>
                <body>
                    <rapi-doc
                        spec-url="{}"
                        theme="light"
                        show-header="false"
                    ></rapi-doc>
                </body>
                </html>
                "#,
                OPENAPI_ENDPOINT
            ))
        }))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": time::OffsetDateTime::now_utc().to_string()
    }))
}
