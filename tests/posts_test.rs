use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::json;
use tower::ServiceExt;

use crate::common::test_app;

mod common;

#[tokio::test]
async fn create_post_without_token_returns_401() {
    let app = test_app().await;

    let request = Request::builder()
        .uri("/posts")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "title": "테스트 글",
                "content": "본문입니다"
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nonexistent_post_returns_404() {
    let app = test_app().await;

    let request = Request::builder()
        .uri("/posts/9999")
        .method(Method::GET)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
