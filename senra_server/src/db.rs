use crate::config::Config;
use crate::errors::{AppError, Result};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self> {
        let pool = SqlitePool::connect(&config.database.url)
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(self.pool())
            .await
            .unwrap();
        Ok(())
    }
}
