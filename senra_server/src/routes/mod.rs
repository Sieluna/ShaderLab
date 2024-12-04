mod auth;
//  notebook;

use axum::Router;
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(auth::router(state.clone()))
        // .merge(notebook::router(state.clone()))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_headers(Any)
                .allow_origin(Any),
        )
}
