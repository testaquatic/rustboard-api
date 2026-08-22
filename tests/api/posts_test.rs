use axum::http::StatusCode;
use rustboard_api::domain::post::CreatePostInput;
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

    let response = ctx.app().oneshot(common::get("/posts/1")).await.unwrap();

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

#[tokio::test]
async fn list_returns_seeded_posts() {
    let ctx = common::TestContext::new().await;

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
    let response = ctx.app().oneshot(common::get("/posts")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = common::response_json(response).await;
    let posts = json["items"].as_array().unwrap();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0]["title"], "두 번째");
    assert_eq!(posts[1]["title"], "첫 번째");
}

#[tokio::test]
async fn owner_can_delete_own_post() {
    // 회원가입과 로그인
    let ctx = common::TestContext::new().await;
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

    let json = common::response_json(response.pop().unwrap()).await;
    let post_id = json["id"].as_i64().unwrap();

    // 삭제
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::delete(&format!("/posts/{}", post_id)),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제 확인
    let response = ctx
        .app()
        .oneshot(common::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
