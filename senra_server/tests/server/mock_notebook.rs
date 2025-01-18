use senra_server::{CreateNotebook, Notebook, Result};
use time::OffsetDateTime;

use crate::server::MockServer;

pub struct NotebookOptions {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub visibility: String,
    pub content: serde_json::Value,
}

impl NotebookOptions {
    pub fn new() -> Self {
        Self {
            title: "Test Notebook".to_string(),
            description: "Test notebook description".to_string(),
            tags: vec!["test".to_string()],
            visibility: "public".to_string(),
            content: serde_json::json!({
                "cells": []
            }),
        }
    }
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_visibility(mut self, visibility: &str) -> Self {
        self.visibility = visibility.to_string();
        self
    }

    pub fn with_content(mut self, content: serde_json::Value) -> Self {
        self.content = content;
        self
    }
}

impl MockServer {
    pub async fn create_notebook(
        &mut self,
        user_id: i64,
        options: NotebookOptions,
    ) -> Result<Notebook> {
        let notebook_service = self.state.services.notebook.clone();

        let new_notebook = CreateNotebook {
            title: options.title,
            description: Some(options.description),
            content: options.content,
            tags: options.tags,
            preview: None,
            visibility: options.visibility,
        };

        notebook_service
            .create_notebook(user_id, new_notebook)
            .await
    }

    pub async fn create_notebook_with_stats(
        &mut self,
        user_id: i64,
        options: NotebookOptions,
        updated_at: OffsetDateTime,
        view_count: i64,
        like_count: i64,
        comment_count: i64,
    ) -> Result<Notebook> {
        let notebook = self.create_notebook(user_id, options).await?;

        self.update_notebook_stats(
            notebook.id,
            view_count,
            like_count,
            comment_count,
            updated_at,
        )
        .await?;

        Ok(notebook)
    }

    async fn update_notebook_stats(
        &self,
        notebook_id: i64,
        view_count: i64,
        like_count: i64,
        comment_count: i64,
        updated_at: OffsetDateTime,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE notebook_stats
            SET 
                view_count = $1,
                like_count = $2,
                comment_count = $3,
                updated_at = $4
            WHERE notebook_id = $5
            "#,
        )
        .bind(view_count)
        .bind(like_count)
        .bind(comment_count)
        .bind(updated_at)
        .bind(notebook_id)
        .execute(self.get_db().pool())
        .await?;

        Ok(())
    }
}
