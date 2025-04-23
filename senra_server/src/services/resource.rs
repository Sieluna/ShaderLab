use sqlx::{QueryBuilder, SqlitePool};

use crate::errors::{NotebookError, Result};
use crate::models::{CreateResource, Resource, UpdateResource};

#[derive(Clone)]
pub struct ResourceService {
    pool: SqlitePool,
}

impl ResourceService {
    pub fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create_resource(
        &self,
        user_id: i64,
        create_resource: CreateResource,
    ) -> Result<Resource> {
        // Verify notebook ownership
        let notebook_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM notebooks
                WHERE id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(create_resource.notebook_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        if !notebook_exists {
            return Err(NotebookError::NotFound.into());
        }

        let resource: Resource = sqlx::query_as(
            r#"
            INSERT INTO resources (notebook_id, name, resource_type, data, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(create_resource.notebook_id)
        .bind(create_resource.name)
        .bind(create_resource.resource_type)
        .bind(create_resource.data)
        .bind(create_resource.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(resource)
    }

    pub async fn get_resource(&self, user_id: i64, id: i64) -> Result<Resource> {
        let resource: Option<Resource> = sqlx::query_as(
            r#"
            SELECT r.* FROM resources r
            JOIN notebooks n ON r.notebook_id = n.id
            WHERE r.id = $1 AND n.user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(resource.ok_or(NotebookError::NotFound)?)
    }

    pub async fn get_resources(&self, notebook_id: i64) -> Result<Vec<Resource>> {
        let resources: Vec<Resource> = sqlx::query_as(
            r#"
            SELECT * FROM resources WHERE notebook_id = $1
            "#,
        )
        .bind(notebook_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(resources)
    }

    pub async fn update_resource(
        &self,
        user_id: i64,
        id: i64,
        update_resource: UpdateResource,
    ) -> Result<Resource> {
        let mut query_builder = QueryBuilder::new("UPDATE resources SET ");
        let mut has_changes = false;

        if let Some(name) = &update_resource.name {
            query_builder.push("name = ").push_bind(name);
            has_changes = true;
        }

        if let Some(data) = &update_resource.data {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("data = ").push_bind(data);
            has_changes = true;
        }

        if let Some(metadata) = &update_resource.metadata {
            if has_changes {
                query_builder.push(", ");
            }
            query_builder.push("metadata = ").push_bind(metadata);
            has_changes = true;
        }

        if !has_changes {
            return Err(NotebookError::NoChanges.into());
        }

        query_builder.push(
            r#"
            WHERE id = $1 AND notebook_id IN (
                SELECT id FROM notebooks WHERE user_id = $2
            )
            RETURNING *
            "#,
        );

        let resource = query_builder
            .build_query_as::<Resource>()
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(resource.ok_or(NotebookError::NotFound)?)
    }

    pub async fn delete_resource(&self, user_id: i64, id: i64) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM resources
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
            return Err(NotebookError::NotFound.into());
        }

        Ok(())
    }
}
