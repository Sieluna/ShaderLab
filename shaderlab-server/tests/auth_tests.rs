use axum::Router;
use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::{Service, ServiceExt};

use shaderlab_server::{config::Config, db::Database, routes::create_router, state::AppState};

async fn app() -> Router {
    let config = Config::new();

    let db = Database::new(&config).await.unwrap();
    db.run_migrations().await.unwrap();

    create_router(AppState::new(config, db))
}

#[tokio::test]
async fn test_auth_workflow() {
    let mut app = app().await.into_service();

    // First register a user
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
                        "username": "loginuser",
                        "email": "login@example.com",
                        "password": "password123"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Then login to get a token
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::POST)
                .uri("/auth/login")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "username": "loginuser",
                        "password": "password123"
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

    let user = body.get("user").unwrap();
    assert_eq!(user["username"], "loginuser");
    assert_eq!(user["email"], "login@example.com");
    assert!(user.get("id").is_some());

    let token = body.get("token").unwrap().as_str().unwrap();
    assert!(!token.is_empty());

    // Test editing user
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::PATCH)
                .uri("/auth/edit")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "username": "newusername",
                        "email": "newemail@example.com"
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
    assert_eq!(body["username"], "newusername");
    assert_eq!(body["email"], "newemail@example.com");
}
