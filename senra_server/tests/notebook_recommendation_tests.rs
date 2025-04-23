mod server;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use server::{MockServer, NotebookOptions};
use time::OffsetDateTime;
use tower::{Service, ServiceExt};

#[tokio::test]
async fn test_notebook_recommendation_algorithm() {
    let mut server = MockServer::new().await;
    let mut app = server.into_service();

    // Create test user and notebook
    let user = server
        .create_user("test_user", "test_user@test.com", "test_password")
        .await
        .unwrap();
    let token = server.create_token(user.id).await.unwrap();

    let _notebooks = vec![
        server
            .create_notebook_with_stats(
                user.id,
                NotebookOptions::new()
                    .with_title("High Engagement Notebook")
                    .with_description("A notebook with high engagement"),
                OffsetDateTime::now_utc(),
                1000, // views
                100,  // likes
                30,   // comments
            )
            .await
            .unwrap(),
        server
            .create_notebook_with_stats(
                user.id,
                NotebookOptions::new()
                    .with_title("New Notebook")
                    .with_description("A newly created notebook"),
                OffsetDateTime::now_utc(),
                100, // views
                10,  // likes
                3,   // comments
            )
            .await
            .unwrap(),
        server
            .create_notebook_with_stats(
                user.id,
                NotebookOptions::new()
                    .with_title("Quality Content")
                    .with_description("An older notebook with high quality content")
                    .with_tags(vec!["quality", "content"]),
                OffsetDateTime::now_utc() - time::Duration::days(10),
                500, // views
                80,  // likes
                20,  // comments
            )
            .await
            .unwrap(),
    ];

    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri("/notebooks?page=1&per_page=10")
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let notebooks = body["notebooks"].as_array().unwrap();
    let total = body["total"].as_i64().unwrap();

    assert_eq!(total, 3);
    assert_eq!(notebooks.len(), 3);

    assert_eq!(notebooks[0]["title"], "High Engagement Notebook");
    assert_eq!(notebooks[1]["title"], "Quality Content");
    assert_eq!(notebooks[2]["title"], "New Notebook");
}
