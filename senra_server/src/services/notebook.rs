use sqlx::{QueryBuilder, SqlitePool};

use crate::errors::{NotebookError, Result};
use crate::models::*;

#[derive(Clone)]
pub struct NotebookService {
    pool: SqlitePool,
}

impl NotebookService {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn get_notebook_tags(&self, notebook_id: i64) -> Result<Vec<NotebookTag>> {
        let tags: Vec<NotebookTag> = sqlx::query_as(
            r#"
            SELECT * FROM notebook_tags
            WHERE notebook_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    pub async fn create_notebook_tag(&self, notebook_id: i64, tag: String) -> Result<NotebookTag> {
        let tag = sqlx::query_as(
            r#"
            INSERT INTO notebook_tags (notebook_id, tag)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(notebook_id)
        .bind(tag)
        .fetch_one(&self.pool)
        .await?;

        Ok(tag)
    }

    pub async fn get_notebook_stats(&self, notebook_id: i64) -> Result<NotebookStats> {
        let stats: NotebookStats = sqlx::query_as(
            r#"
            SELECT * FROM notebook_stats
            WHERE notebook_id = $1
            "#,
        )
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn create_notebook_stats(&self, notebook_id: i64) -> Result<NotebookStats> {
        let stats: NotebookStats = sqlx::query_as(
            r#"
            INSERT INTO notebook_stats (notebook_id)
            VALUES ($1)
            RETURNING *
            "#,
        )
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    pub async fn is_notebook_liked(&self, user_id: i64, notebook_id: i64) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM notebook_likes
            WHERE user_id = $1 AND notebook_id = $2
            "#,
        )
        .bind(user_id)
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
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

        let total = sqlx::query_scalar("SELECT COUNT(*) FROM notebooks WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok((notebooks, total))
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

    pub async fn create_notebook(
        &self,
        user_id: i64,
        create_notebook: CreateNotebook,
    ) -> Result<Notebook> {
        let notebook: Notebook = sqlx::query_as(
            r#"
            INSERT INTO notebooks (user_id, title, description, content, preview, visibility)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(create_notebook.title)
        .bind(create_notebook.description)
        .bind(create_notebook.content)
        .bind(create_notebook.preview)
        .bind(create_notebook.visibility)
        .fetch_one(&self.pool)
        .await?;

        Ok(notebook)
    }

    pub async fn update_notebook(
        &self,
        user_id: i64,
        id: i64,
        update_notebook: UpdateNotebook,
    ) -> Result<Notebook> {
        let mut query_builder = QueryBuilder::new("UPDATE notebooks SET ");

        let mut has_changes = false;

        if let Some(title) = &update_notebook.title {
            query_builder.push("title = ").push_bind(title);
            has_changes = true;
        }

        if let Some(description) = &update_notebook.description {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("description = ").push_bind(description);
            has_changes = true;
        }

        if let Some(content) = &update_notebook.content {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("content = ").push_bind(content);
            has_changes = true;

            sqlx::query(
                r#"
                INSERT INTO notebook_versions (notebook_id, user_id, version, content)
                SELECT id, user_id, version, content
                FROM notebooks
                WHERE id = $1
                "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await?;
        }

        if let Some(preview) = &update_notebook.preview {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("preview = ").push_bind(preview);
            has_changes = true;
        }

        if let Some(visibility) = &update_notebook.visibility {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("visibility = ").push_bind(visibility);
            has_changes = true;
        }

        if !has_changes {
            Err(NotebookError::NoChanges)?;
        }

        query_builder
            .push(", updated_at = datetime('now') WHERE id = ")
            .push_bind(id)
            .push(" AND user_id = ")
            .push_bind(user_id)
            .push(" RETURNING *");

        let notebook = query_builder
            .build_query_as::<Notebook>()
            .fetch_optional(&self.pool)
            .await?
            .ok_or(NotebookError::NotFound)?;

        if let Some(tags) = &update_notebook.tags {
            sqlx::query(
                r#"
                DELETE FROM notebook_tags
                WHERE notebook_id = $1
                "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await?;

            for tag in tags {
                self.create_notebook_tag(id, tag.clone()).await?;
            }
        }

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

        let total = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM notebook_versions
            WHERE notebook_id = $1
            "#,
        )
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((versions, total))
    }

    pub async fn list_comments(
        &self,
        notebook_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<NotebookComment>, i64)> {
        let offset = (page - 1) * per_page;

        let comments: Vec<NotebookComment> = sqlx::query_as(
            r#"
            SELECT * FROM notebook_comments
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

        let total = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM notebook_comments
            WHERE notebook_id = $1
            "#,
        )
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((comments, total))
    }

    pub async fn create_comment(
        &self,
        user_id: i64,
        notebook_id: i64,
        content: String,
    ) -> Result<NotebookComment> {
        let comment: NotebookComment = sqlx::query_as(
            r#"
            INSERT INTO notebook_comments (notebook_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(notebook_id)
        .bind(user_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    pub async fn delete_comment(
        &self,
        user_id: i64,
        notebook_id: i64,
        comment_id: i64,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM notebook_comments
            WHERE id = $1 AND notebook_id = $2 AND user_id = $3
            "#,
        )
        .bind(comment_id)
        .bind(notebook_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(NotebookError::NotFound.into());
        }

        Ok(())
    }
}
