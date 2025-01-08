mod server;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::MockServer;
use tower::{Service, ServiceExt};

#[tokio::test]
async fn test_notebook_workflow() {
    let mut server = MockServer::new().await;

    let mut app = server.into_service();

    let user = server.create_test_user("test_user", "test_password").await;
    let token = server.create_test_token(user.id).await;

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
                .uri(format!("/user/{}", user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["notebooks"]["notebooks"].as_array().unwrap().len(), 1);
    assert_eq!(body["notebooks"]["total"], 1);

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
