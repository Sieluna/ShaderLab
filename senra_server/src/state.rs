use std::sync::Arc;

use crate::config::Config;
use crate::db::Database;
use crate::services::*;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<Database>,
    pub services: Services,
}

#[derive(Clone)]
pub struct Services {
    pub auth: AuthService,
    pub notebook: NotebookService,
    pub resource: ResourceService,
    pub shader: ShaderService,
    pub user: UserService,
}

impl AppState {
    pub fn new(config: Config, db: Database) -> Self {
        let config = Arc::new(config);
        let db = Arc::new(db);

        let services = Services {
            auth: AuthService::new(db.pool(), &config.auth.jwt_secret),
            notebook: NotebookService::new(db.pool()),
            resource: ResourceService::new(db.pool()),
            shader: ShaderService::new(db.pool()),
            user: UserService::new(db.pool()),
        };

        Self {
            config,
            db,
            services,
        }
    }
}
