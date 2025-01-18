#![allow(dead_code)]

mod mock_notebook;
mod mock_user;

use axum::Router;
use axum::body::Body;
use axum::routing::RouterIntoService;
use senra_server::{AppState, Config, Database, create_router};

pub use mock_notebook::NotebookOptions;

pub struct MockServer {
    pub app: Router,
    state: AppState,
}

impl MockServer {
    pub async fn new() -> Self {
        let config = Config::new();

        let db = Database::new(&config).await.unwrap();
        db.run_migrations().await.unwrap();

        let state = AppState::new(config, db);
        let app = create_router(state.clone());

        Self { app, state }
    }

    pub fn into_service(&self) -> RouterIntoService<Body> {
        self.app.clone().into_service()
    }

    pub fn get_db(&self) -> &Database {
        &self.state.db
    }
}
