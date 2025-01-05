mod server;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server::MockServer;
use tower::{Service, ServiceExt};

#[tokio::test]
async fn test_notebook_comment_routes() {
    let mut server = MockServer::new().await;
    let mut app = server.into_service();

    // 创建测试用户和笔记本
    let user = server.create_test_user("test_user", "test_password").await;
    let token = server.create_test_token(user.id).await;
    let notebook = server.create_test_notebook(user.id).await;

    // 测试创建评论
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("/notebooks/{}/comments", notebook.id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "content": "This is a test comment"
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
    let comment_id = body["id"].as_i64().unwrap();

    // 测试获取评论列表
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/notebooks/{}/comments", notebook.id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["comments"].as_array().unwrap().len(), 1);
    assert_eq!(body["comments"][0]["content"], "This is a test comment");

    // 测试删除评论
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::DELETE)
                .uri(format!(
                    "/notebooks/{}/comments/{}",
                    notebook.id, comment_id
                ))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 验证评论已被删除
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(
            Request::builder()
                .method(http::Method::GET)
                .uri(format!("/notebooks/{}/comments", notebook.id))
                .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["comments"].as_array().unwrap().len(), 0);
}
