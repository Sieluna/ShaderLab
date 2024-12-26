use axum::Router;
use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use axum::routing::RouterIntoService;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::{Service, ServiceExt};

use senra_server::{config::Config, db::Database, routes::create_router, state::AppState};

async fn app() -> Router {
    let config = Config::new();

    let db = Database::new(&config).await.unwrap();
    db.run_migrations().await.unwrap();

    create_router(AppState::new(config, db))
}

async fn generate_token(mut app: &mut RouterIntoService<Body>) -> String {
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/register")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "username": "test_user",
                        "email": "test_user@example.com",
                        "password": "test_password"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    body.get("token").unwrap().as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_notebook_workflow() {
    let mut app = app().await.into_service();

    let token = {
        let mut app = &mut app;
        generate_token(&mut app).await
    };

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::POST)
                .uri("/notebooks")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "title": "Test Notebook",
                        "description": "This is a test notebook",
                        "content": {
                            "cells": []
                        },
                        "tags": ["test", "rust"],
                        "visibility": "public"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let notebook_id = body["id"].as_i64().unwrap();

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri("/notebooks")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["notebooks"].as_array().unwrap().len(), 1);
    assert_eq!(body["total"], 1);

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/notebooks/{}", notebook_id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["title"], "Test Notebook");
    assert_eq!(body["description"], "This is a test notebook");
    assert_eq!(body["visibility"], "public");

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::PATCH)
                .uri(format!("/notebooks/{}", notebook_id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "title": "Updated Notebook",
                        "description": "This is an updated test notebook",
                        "tags": ["test", "rust", "update"]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["title"], "Updated Notebook");
    assert_eq!(body["description"], "This is an updated test notebook");

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!("/notebooks/{}", notebook_id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/notebooks/{}", notebook_id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
