use axum::http::StatusCode;
use insta::assert_json_snapshot;
use rustboard_api::domain::post::CreatePostInput;
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn snapshot_create_post_response() {
    let ctx = common::TestContext::new().await;

    // 회원가입과 로그인
    let token = ctx.signup_and_login().await.unwrap();

    // 글 작성
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::post_json(
                "/posts",
                &json!({
                    "title": "스냅샷 테스트 글",
                    "content": "이 응답 구조를 고정합니다"
                }),
            ),
            &token,
        ))
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), StatusCode::CREATED);

    let json = common::response_json(response).await;

    // 변동 필드를 redaction으로 치환
    assert_json_snapshot!(json, {
        ".author_id" => "[author_id]",
        ".id" => "[id]",
        ".user_id" => "[id]",
        ".created_at" => "[datetime]",
        ".updated_at" => "[datetime]",
    });
}

#[tokio::test]
async fn snapshot_unauthorized_error() {
    let ctx = common::TestContext::new().await;

    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/posts",
            &json!(
                {
                    "title": "No Token",
                    "content": "실패",
                }
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json = common::response_json(response).await;
    assert_json_snapshot!("error_unauthorized", json);
}

#[tokio::test]
async fn snapshot_not_found_error() {
    let ctx = common::TestContext::new().await;

    let response = ctx
        .app()
        .oneshot(common::get("/posts/999999"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = common::response_json(response).await;
    assert_json_snapshot!("error_not_found", json);
}

#[tokio::test]
async fn snapshot_list_posts() {
    let ctx = common::TestContext::new().await;
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
    let response = ctx.app().oneshot(common::get("/posts")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = common::response_json(response).await;
    assert_json_snapshot!(json, {
        ".items[].author_id" => "[author_id]",
        ".items[].id" => "[id]",
        ".items[].created_at" => "[datetime]",
        ".items[].updated_at" => "[datetime]",
        ".next_cursor" => "[next_cursor]",
    });
}
