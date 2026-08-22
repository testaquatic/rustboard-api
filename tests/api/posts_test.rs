use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use crate::common::{self, TestContext};

#[tokio::test]
async fn create_post_without_token_returns_401() {
    let app = TestContext::new().await.app();

    let request = common::post_json(
        "/posts",
        &json!({
            "title": "테스트 글",
            "content": "본문입니다",
        }),
    );
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nonexistent_post_returns_404() {
    let app = TestContext::new().await.app();

    let request = common::get("/posts/9999");
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_post_returns_correct_fields() {
    let app = TestContext::new().await.app();

    let response = app.oneshot(common::get("/posts/1")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let response_json = common::response_json(response).await;
    assert_eq!(response_json["title"], "첫 번째 글");
    assert!(response_json["id"].is_number());
    assert!(response_json["created_at"].is_string());
}

#[tokio::test]
async fn signup_then_login_then_create_post() {
    let ctx = common::TestContext::new().await;

    // 회원가입
    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/signup",
            &json!({
                "email": "alice@test.com",
                "password": "pass1234",
                "display_name": "Alice",
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 로그인
    let login_response = ctx
        .app()
        .oneshot(common::post_json(
            "/login",
            &json!({
                "email": "alice@test.com",
                "password": "pass1234",
            }),
        ))
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
    let json = common::response_json(login_response).await;
    let token = json["token"].as_str().unwrap();

    // 글 작성
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::post_json(
                "/posts",
                &json!({
                    "title": "Alice의 첫 글", "content": "테스트입니다"
                }),
            ),
            token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn list_returns_empty_when_no_posts() {
    let ctx = common::TestContext::new().await;

    let response = ctx.app().oneshot(common::get("/posts")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response_json = common::response_json(response).await;
    let posts = response_json["items"].as_array().unwrap();
    assert!(posts.is_empty());
}
