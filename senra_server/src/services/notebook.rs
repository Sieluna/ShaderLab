use sqlx::SqlitePool;

use crate::errors::{NotebookError, Result};
use crate::models::{Notebook, NotebookVersion};

#[derive(Clone)]
pub struct NotebookService {
    pool: SqlitePool,
}

impl NotebookService {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn list_notebooks(
        &self,
        user_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<Notebook>, i64)> {
        let offset = (page - 1) * per_page;

        let notebooks: Vec<Notebook> = sqlx::query_as(
            r#"
            SELECT * FROM notebooks
            WHERE user_id = $1
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM notebooks
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok((notebooks, total))
    }

    pub async fn create_notebook(
        &self,
        user_id: i64,
        title: String,
        content: serde_json::Value,
    ) -> Result<Notebook> {
        let notebook: Notebook = sqlx::query_as(
            r#"
            INSERT INTO notebooks (user_id, title, content)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(title)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(notebook)
    }

    pub async fn get_notebook(&self, user_id: i64, id: i64) -> Result<Notebook> {
        let notebook: Notebook = sqlx::query_as(
            r#"
            SELECT * FROM notebooks
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(NotebookError::NotFound)?;

        Ok(notebook)
    }

    pub async fn update_notebook(
        &self,
        user_id: i64,
        id: i64,
        title: Option<String>,
        content: Option<serde_json::Value>,
    ) -> Result<Notebook> {
        let notebook: Notebook = sqlx::query_as(
            r#"
            UPDATE notebooks
            SET
                title = COALESCE($1, title),
                content = COALESCE($2, content),
                updated_at = CURRENT_TIMESTAMP,
                version = version + 1
            WHERE id = $3 AND user_id = $4
            RETURNING *
            "#,
        )
        .bind(title)
        .bind(content)
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(NotebookError::NotFound)?;

        Ok(notebook)
    }

    pub async fn delete_notebook(&self, user_id: i64, id: i64) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM notebooks
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            Err(NotebookError::NotFound)?;
        }

        Ok(())
    }

    pub async fn list_versions(
        &self,
        notebook_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<NotebookVersion>, i64)> {
        let offset = (page - 1) * per_page;

        let versions: Vec<NotebookVersion> = sqlx::query_as(
            r#"
            SELECT * FROM notebook_versions
            WHERE notebook_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(notebook_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM notebook_versions
            WHERE notebook_id = $1
            "#,
            notebook_id
        )
        .fetch_one(&self.pool)
        .await?
        .count;

        Ok((versions, total))
    }
}
