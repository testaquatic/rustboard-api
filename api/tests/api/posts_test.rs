use axum::http::StatusCode;
use rustboard_api::handler::types::input::CreatePostInput;
use serde_json::json;

use crate::common::test_context::{TestContext, response_json};

#[tokio::test]
async fn create_post_without_token_returns_401() {
    let app = TestContext::new().await;

    let response = app
        .post_json(
            "/posts",
            None,
            &json!({
                "title": "테스트 글",
                "content": "본문입니다",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nonexistent_post_returns_404() {
    let app = TestContext::new().await;

    let (status_code, _) = app.get("/posts/9999", None).await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_post_returns_correct_fields() {
    let ctx = TestContext::new().await;
    let token = ctx.signup_and_login().await.unwrap();
    ctx.seed_post(
        &vec![CreatePostInput {
            title: "첫 번째 글".to_string(),
            content: "본문입니다".to_string(),
        }],
        &token,
    )
    .await;

    let (status_code, json_body) = ctx.get("/posts/1", None).await;

    assert_eq!(status_code, StatusCode::OK);
    assert_eq!(json_body["title"], "첫 번째 글");
    assert!(json_body["id"].is_number());
    assert!(json_body["created_at"].is_string());
}

#[tokio::test]
async fn signup_then_login_then_create_post() {
    let ctx = TestContext::new().await;

    // 회원가입
    let response = ctx
        .post_json(
            "/signup",
            None,
            &json!({
                "email": "alice@test.com",
                "password": "pass1234",
                "display_name": "Alice",
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);

    // 로그인
    let response = ctx
        .post_json(
            "/login",
            None,
            &json!({
                "email": "alice@test.com",
                "password": "pass1234",
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let json_body = response_json(response).await;
    let token = json_body["token"].as_str().unwrap();

    // 글 작성
    let response = ctx
        .post_json(
            "/posts",
            Some(token),
            &json!({
                "title": "Alice의 첫 글", "content": "테스트입니다"
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn list_returns_empty_when_no_posts() {
    let ctx = TestContext::new().await;

    let (status_code, json_body) = ctx.get("/posts", None).await;

    assert_eq!(status_code, StatusCode::OK);
    let posts = json_body["items"].as_array().unwrap();
    assert!(posts.is_empty());
}

#[tokio::test]
async fn list_returns_seeded_posts() {
    let ctx = TestContext::new().await;

    // 회원 가입과 로그인
    let token = ctx.signup_and_login().await.unwrap();

    // 사전 데이터 주입
    let seed_posts = vec![
        CreatePostInput {
            title: "첫 번째".to_string(),
            content: "내용1".to_string(),
        },
        CreatePostInput {
            title: "두 번째".to_string(),
            content: "내용2".to_string(),
        },
    ];
    ctx.seed_post(&seed_posts, &token).await;

    // 글 목록 조회
    let (status_code, json_body) = ctx.get("/posts", None).await;

    assert_eq!(status_code, StatusCode::OK);
    let posts = json_body["items"].as_array().unwrap();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0]["title"], "두 번째");
    assert_eq!(posts[1]["title"], "첫 번째");
}

#[tokio::test]
async fn owner_can_delete_own_post() {
    // 회원가입과 로그인
    let ctx = TestContext::new().await;
    let token = ctx.signup_and_login().await.unwrap();

    // 글 작성
    let mut response = ctx
        .seed_post(
            &vec![CreatePostInput {
                title: "삭제 테스트".to_string(),
                content: "곧 지워질 글".to_string(),
            }],
            &token,
        )
        .await;
    assert_eq!(response.len(), 1);

    let response = response.pop().expect("응답이 없음");
    let json_body = response_json(response).await;
    let post_id = json_body["id"].as_i64().unwrap();

    // 삭제
    let response = ctx
        .delete(&format!("/posts/{}", post_id), Some(&token))
        .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제 확인
    let (status_code, _) = ctx.get(&format!("/posts/{}", post_id), None).await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}
