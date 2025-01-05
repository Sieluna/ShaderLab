use crate::server::MockServer;

use senra_server::models::{CreateNotebook, Notebook};

impl MockServer {
    pub async fn create_test_notebook(&mut self, user_id: i64) -> Notebook {
        let notebook_service = self.state.services.notebook.clone();

        let new_notebook = CreateNotebook {
            title: "Test Notebook".to_string(),
            description: Some("This is a test notebook".to_string()),
            content: serde_json::json!({
                "cells": []
            }),
            tags: vec!["test".to_string(), "rust".to_string()],
            preview: None,
            visibility: "public".to_string(),
        };

        notebook_service
            .create_notebook(user_id, new_notebook)
            .await
            .unwrap()
    }

    pub async fn create_test_notebook_with_visibility(
        &mut self,
        user_id: i64,
        visibility: &str,
    ) -> Notebook {
        let notebook_service = self.state.services.notebook.clone();

        let new_notebook = CreateNotebook {
            title: "Test Notebook".to_string(),
            description: Some("This is a test notebook".to_string()),
            content: serde_json::json!({
                "cells": []
            }),
            tags: vec!["test".to_string(), "rust".to_string()],
            preview: None,
            visibility: visibility.to_string(),
        };

        notebook_service
            .create_notebook(user_id, new_notebook)
            .await
            .unwrap()
    }
}
