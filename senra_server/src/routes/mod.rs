mod auth;
mod notebook;
mod user;
mod ws;

use axum::Router;
use axum::http::Method;
use axum::response::Json;
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
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_headers(Any)
                .allow_origin(Any),
        )
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": time::OffsetDateTime::now_utc().to_string()
    }))
}
