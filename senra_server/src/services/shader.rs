use sqlx::{QueryBuilder, SqlitePool};

use crate::errors::{NotebookError, Result, ShaderError};
use crate::models::{CreateShader, Shader, ShaderVersion, UpdateShader};

#[derive(Clone)]
pub struct ShaderService {
    pool: SqlitePool,
}

impl ShaderService {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create_shader(&self, user_id: i64, create_shader: CreateShader) -> Result<Shader> {
        let mut tx = self.pool.begin().await?;

        // Verify notebook ownership
        let notebook_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM notebooks
                WHERE id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(create_shader.notebook_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        if !notebook_exists {
            return Err(NotebookError::NotFound.into());
        }

        let shader: Shader = sqlx::query_as(
            r#"
            INSERT INTO shaders (notebook_id, name, shader_type, code)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(create_shader.notebook_id)
        .bind(create_shader.name)
        .bind(create_shader.shader_type)
        .bind(create_shader.code.clone())
        .fetch_one(&mut *tx)
        .await?;

        // Create initial version
        sqlx::query(
            r#"
            INSERT INTO shader_versions (shader_id, version, code)
            VALUES ($1, 1, $2)
            "#,
        )
        .bind(shader.id)
        .bind(create_shader.code)
        .execute(&mut *tx)
        .await?;

        Ok(shader)
    }

    pub async fn get_shaders(&self, notebook_id: i64) -> Result<Vec<Shader>> {
        let shaders = sqlx::query_as(
            r#"
            SELECT * FROM shaders
            WHERE notebook_id = $1
            "#,
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(shaders)
    }

    pub async fn get_shader(&self, user_id: i64, id: i64) -> Result<Shader> {
        let shader: Option<Shader> = sqlx::query_as(
            r#"
            SELECT s.* FROM shaders s
            JOIN notebooks n ON s.notebook_id = n.id
            WHERE s.id = $1 AND n.user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(shader.ok_or(ShaderError::NotFound)?)
    }

    pub async fn update_shader(
        &self,
        user_id: i64,
        id: i64,
        update_shader: UpdateShader,
    ) -> Result<Shader> {
        let mut tx = self.pool.begin().await?;

        let mut query_builder = QueryBuilder::new("UPDATE shaders SET ");
        let mut has_changes = false;

        if let Some(name) = &update_shader.name {
            query_builder.push("name = ").push_bind(name);
            has_changes = true;
        }

        if let Some(shader_type) = &update_shader.shader_type {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("shader_type = ").push_bind(shader_type);
            has_changes = true;
        }

        if let Some(code) = &update_shader.code {
            if has_changes {
                query_builder.push(", ");
            }

            // Get current version and increment
            let current_version: i64 = sqlx::query_scalar(
                r#"
                SELECT COALESCE(MAX(version), 0) FROM shader_versions
                WHERE shader_id = $1
                "#,
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            let update_version = current_version + 1;

            sqlx::query(
                r#"
                INSERT INTO shader_versions (shader_id, version, code)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(id)
            .bind(update_version)
            .bind(code)
            .execute(&mut *tx)
            .await?;

            query_builder
                .push("code = ")
                .push_bind(code)
                .push(", version = ")
                .push_bind(update_version);
            has_changes = true;
        }

        if !has_changes {
            return Err(ShaderError::NoChanges.into());
        }

        query_builder.push(
            r#"
            WHERE id = $1 AND notebook_id IN (
                SELECT id FROM notebooks WHERE user_id = $2
            )
            RETURNING *
            "#,
        );

        let shader = query_builder
            .build_query_as::<Shader>()
            .bind(id)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;

        Ok(shader.ok_or(ShaderError::NotFound)?)
    }

    pub async fn delete_shader(&self, user_id: i64, id: i64) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM shaders
            WHERE id = $1 AND notebook_id IN (
                SELECT id FROM notebooks WHERE user_id = $2
            )
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ShaderError::NotFound.into());
        }

        Ok(())
    }

    pub async fn list_versions(
        &self,
        shader_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<ShaderVersion>, i64)> {
        // Check if shader exists
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM shaders
                WHERE id = $1
            )
            "#,
        )
        .bind(shader_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            Err(NotebookError::NotFound)?;
        }

        let offset = (page - 1) * per_page;

        let versions = sqlx::query_as(
            r#"
            SELECT * FROM shader_versions
            WHERE shader_id = $1
            ORDER BY version DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(shader_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM shader_versions
            WHERE shader_id = $1
            "#,
        )
        .bind(shader_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((versions, total))
    }
}
