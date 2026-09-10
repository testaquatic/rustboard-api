use axum::http::StatusCode;
use insta::assert_json_snapshot;
use rustboard_api::domain::post::CreatePostInput;
use serde_json::json;

use crate::common::test_context::{TestContext, response_json};

#[tokio::test]
async fn snapshot_create_post_response() {
    let ctx = TestContext::new().await;

    // 회원가입과 로그인
    let token = ctx.signup_and_login().await.unwrap();

    // 글 작성
    let response = ctx
        .post_json(
            "/posts",
            Some(&token),
            &json!({
                "title": "스냅샷 테스트 글",
                "content": "이 응답 구조를 고정합니다"
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let json_body = response_json(response).await;

    // 변동 필드를 redaction으로 치환
    assert_json_snapshot!(json_body, {
        ".author_id" => "[author_id]",
        ".id" => "[id]",
        ".user_id" => "[id]",
        ".created_at" => "[datetime]",
        ".updated_at" => "[datetime]",
    });
}

#[tokio::test]
async fn snapshot_unauthorized_error() {
    let ctx = TestContext::new().await;

    let response = ctx
        .post_json(
            "/posts",
            None,
            &json!(
                {
                    "title": "No Token",
                    "content": "실패"
                }
            ),
        )
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json_body = response_json(response).await;

    assert_json_snapshot!("error_unauthorized", json_body);
}

#[tokio::test]
async fn snapshot_not_found_error() {
    let ctx = TestContext::new().await;

    let (status, json) = ctx.get("/posts/999999", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_json_snapshot!("error_not_found", json);
}

#[tokio::test]
async fn snapshot_list_posts() {
    let ctx = TestContext::new().await;
    let token = ctx.signup_and_login().await.unwrap();

    // 글 2개 작성
    ctx.seed_post(
        &vec![
            CreatePostInput {
                title: "첫 번째 글".to_string(),
                content: "첫 번째 글 내용".to_string(),
            },
            CreatePostInput {
                title: "두 번째 글".to_string(),
                content: "두 번째 글 내용".to_string(),
            },
        ],
        &token,
    )
    .await;

    // 목록 조회
    let (status, json) = ctx.get("/posts", None).await;
    assert_eq!(status, StatusCode::OK);

    assert_json_snapshot!(json, {
        ".items[].author_id" => "[author_id]",
        ".items[].id" => "[id]",
        ".items[].created_at" => "[datetime]",
        ".items[].updated_at" => "[datetime]",
        ".next_cursor" => "[next_cursor]",
    });
}
