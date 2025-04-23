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

    /// Retrieves all tags associated with a notebook
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

    /// Retrieves statistics for a notebook
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

    /// Checks if a user has liked a notebook
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

    /// Like a notebook
    pub async fn like_notebook(&self, user_id: i64, notebook_id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Check if user has already liked this notebook
        let already_liked = self.is_notebook_liked(user_id, notebook_id).await?;
        if already_liked {
            return Ok(());
        }

        // Add like record
        sqlx::query(
            r#"
            INSERT INTO notebook_likes (notebook_id, user_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(notebook_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Update like count in stats
        sqlx::query(
            r#"
            UPDATE notebook_stats
            SET like_count = like_count + 1
            WHERE notebook_id = $1
            "#,
        )
        .bind(notebook_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Unlike a notebook
    pub async fn unlike_notebook(&self, user_id: i64, notebook_id: i64) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Remove like record
        let result = sqlx::query(
            r#"
            DELETE FROM notebook_likes
            WHERE notebook_id = $1 AND user_id = $2
            "#,
        )
        .bind(notebook_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            // Update like count in stats only if a like was actually removed
            sqlx::query(
                r#"
                UPDATE notebook_stats
                SET like_count = like_count - 1
                WHERE notebook_id = $1
                "#,
            )
            .bind(notebook_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Lists notebooks for a user with pagination
    pub async fn list_notebooks(&self, page: i64, per_page: i64) -> Result<(Vec<Notebook>, i64)> {
        let offset = (page - 1) * per_page;

        // Get recommended notebooks using Bilibili-like recommendation algorithm
        let notebooks: Vec<Notebook> = sqlx::query_as(
                r#"
                WITH notebook_scores AS (
                    SELECT 
                        n.*,
                        -- Base popularity score (weights: views 0.4, likes 0.3, comments 0.3)
                        (s.view_count * 0.4 + s.like_count * 0.3 + s.comment_count * 0.3) as base_score,
                        -- Time decay factor (higher weight for content within 24 hours)
                        CASE 
                            WHEN datetime(n.updated_at) > datetime('now', '-24 hours') THEN 1.5
                            WHEN datetime(n.updated_at) > datetime('now', '-7 days') THEN 1.2
                            ELSE 1.0
                        END as time_factor,
                        -- Content quality factor (based on engagement rate)
                        CASE 
                            WHEN s.view_count > 0 THEN 
                                (s.like_count + s.comment_count) * 1.0 / s.view_count
                            ELSE 0
                        END as quality_factor
                    FROM notebooks n
                    JOIN notebook_stats s ON n.id = s.notebook_id
                    WHERE n.visibility = 'public'
                )
                SELECT * FROM notebook_scores
                ORDER BY 
                    (base_score * time_factor * (1 + quality_factor)) DESC,
                    updated_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        // Get total count
        let total = sqlx::query_scalar(
            r#"
                SELECT COUNT(*) FROM notebooks
                WHERE visibility = 'public'
                "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((notebooks, total))
    }

    /// Lists notebooks for a user with pagination
    pub async fn list_notebooks_by_user(
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

    /// Retrieves a specific notebook by ID
    pub async fn get_notebook(&self, user_id: i64, id: i64) -> Result<Notebook> {
        let notebook: Notebook = sqlx::query_as(
            r#"
            SELECT n.* FROM notebooks n
            WHERE n.id = $1 AND (n.user_id = $2 OR n.visibility = 'public')
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(NotebookError::NotFound)?;

        // Increment view count
        sqlx::query(
            r#"
            UPDATE notebook_stats
            SET view_count = view_count + 1
            WHERE notebook_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(notebook)
    }

    /// Creates a new notebook with all related data in a transaction
    /// This includes:
    /// - Notebook record
    /// - Initial version
    /// - Statistics
    /// - Tags
    pub async fn create_notebook(
        &self,
        user_id: i64,
        create_notebook: CreateNotebook,
    ) -> Result<Notebook> {
        let mut tx = self.pool.begin().await?;

        // Create notebook record
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
        .fetch_one(&mut *tx)
        .await?;

        // Create resources
        for resource in create_notebook.resources {
            sqlx::query(
                r#"
                INSERT INTO resources (notebook_id, name, resource_type, data, metadata)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(notebook.id)
            .bind(resource.name)
            .bind(resource.resource_type)
            .bind(resource.data)
            .bind(resource.metadata)
            .execute(&mut *tx)
            .await?;
        }

        // Create shaders
        for shader in create_notebook.shaders {
            let shader: Shader = sqlx::query_as(
                r#"
                INSERT INTO shaders (notebook_id, name, shader_type, code)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#,
            )
            .bind(notebook.id)
            .bind(shader.name)
            .bind(shader.shader_type)
            .bind(shader.code.clone())
            .fetch_one(&mut *tx)
            .await?;

            // Create initial shader version
            sqlx::query(
                r#"
                INSERT INTO shader_versions (shader_id, version, code)
                VALUES ($1, 1, $2)
                "#,
            )
            .bind(shader.id)
            .bind(shader.code)
            .execute(&mut *tx)
            .await?;
        }

        // Create initial version
        sqlx::query(
            r#"
            INSERT INTO notebook_versions (notebook_id, user_id, version, content)
            VALUES ($1, $2, 1, $3)
            "#,
        )
        .bind(notebook.id)
        .bind(user_id)
        .bind(notebook.content.clone())
        .execute(&mut *tx)
        .await?;

        // Create initial statistics
        sqlx::query(
            r#"
            INSERT INTO notebook_stats (notebook_id)
            VALUES ($1)
            "#,
        )
        .bind(notebook.id)
        .execute(&mut *tx)
        .await?;

        // Add tags
        for tag in create_notebook.tags {
            sqlx::query(
                r#"
                INSERT INTO notebook_tags (notebook_id, tag)
                VALUES ($1, $2)
                "#,
            )
            .bind(notebook.id)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(notebook)
    }

    /// Updates a notebook content and its related data in a transaction
    /// Handles:
    /// - Notebook updates
    /// - Version tracking
    /// - Tag updates
    pub async fn update_notebook(
        &self,
        user_id: i64,
        id: i64,
        update_notebook: UpdateNotebook,
    ) -> Result<Notebook> {
        let mut tx = self.pool.begin().await?;

        let mut query_builder = QueryBuilder::new("UPDATE notebooks SET ");
        let mut has_changes = false;

        // Build update query based on provided fields
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

            // Get current version and increment
            let current_version: i64 = sqlx::query_scalar(
                r#"
                SELECT COALESCE(MAX(version), 0) FROM notebook_versions
                WHERE notebook_id = $1
                "#,
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            let update_version = current_version + 1;

            // Create new version
            sqlx::query(
                r#"
                INSERT INTO notebook_versions (notebook_id, user_id, version, content)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(id)
            .bind(user_id)
            .bind(update_version)
            .bind(content)
            .execute(&mut *tx)
            .await?;

            query_builder
                .push("content = ")
                .push_bind(content)
                .push(", version = ")
                .push_bind(update_version);
            has_changes = true;
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

        if has_changes {
            query_builder
                .push(", updated_at = CURRENT_TIMESTAMP WHERE id = ")
                .push_bind(id)
                .push(" AND user_id = ")
                .push_bind(user_id);

            query_builder
                .push(" RETURNING id, user_id, title, description, content, preview, visibility, version, created_at, updated_at");

            let notebook = query_builder
                .build_query_as::<Notebook>()
                .fetch_one(&mut *tx)
                .await?;

            // Update tags if provided
            if let Some(tags) = &update_notebook.tags {
                sqlx::query(
                    r#"
                    DELETE FROM notebook_tags
                    WHERE notebook_id = $1
                    "#,
                )
                .bind(id)
                .execute(&mut *tx)
                .await?;

                for tag in tags {
                    sqlx::query(
                        r#"
                        INSERT INTO notebook_tags (notebook_id, tag)
                        VALUES ($1, $2)
                        "#,
                    )
                    .bind(id)
                    .bind(tag)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            tx.commit().await?;
            Ok(notebook)
        } else {
            tx.rollback().await?;
            Err(NotebookError::NoChanges)?
        }
    }

    /// Deletes a notebook and all its related data
    /// This includes:
    /// - Tags
    /// - Versions
    /// - Comments
    /// - Statistics
    /// - Likes
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

    /// Lists versions of a notebook with pagination
    pub async fn list_versions(
        &self,
        notebook_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<NotebookVersion>, i64)> {
        // Check if notebook exists
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM notebooks
                WHERE id = $1
            )
            "#,
        )
        .bind(notebook_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            Err(NotebookError::NotFound)?;
        }

        let offset = (page - 1) * per_page;

        let versions: Vec<NotebookVersion> = sqlx::query_as(
            r#"
            SELECT * FROM notebook_versions
            WHERE notebook_id = $1
            ORDER BY version DESC
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

    /// Lists comments for a notebook with pagination
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

    /// Creates a new comment for a notebook
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

    /// Deletes a comment from a notebook
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
            Err(NotebookError::NotFound)?;
        }

        Ok(())
    }
}
