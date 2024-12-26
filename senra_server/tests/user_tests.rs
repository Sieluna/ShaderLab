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
async fn test_user_workflow() {
    let mut app = app().await.into_service();

    let token = generate_token(&mut app).await;

    // Test get user info
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri("/user")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["username"], "test_user");
    assert_eq!(body["email"], "test_user@example.com");

    // Test edit user info
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::PATCH)
                .uri("/user")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "username": "update_user",
                        "email": "update_user@example.com",
                        "password": "update_password"
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
    assert_eq!(body["username"], "update_user");
    assert_eq!(body["email"], "update_user@example.com");

    // Test updated info can login
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
                        "username": "update_user",
                        "password": "update_password"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
