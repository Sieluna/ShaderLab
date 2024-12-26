mod auth;
mod notebook;
mod user;
mod ws;

use axum::Router;
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(auth::router(state.clone()))
        .merge(notebook::router(state.clone()))
        .merge(user::router(state.clone()))
        .merge(ws::router(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_headers(Any)
                .allow_origin(Any),
        )
}
