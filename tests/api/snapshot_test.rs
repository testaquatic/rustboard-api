use axum::http::StatusCode;
use insta::assert_json_snapshot;
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn snapshot_create_post_response() {
    let ctx = common::TestContext::new();

    // 로그인
    let token = ctx.signup_and_login().await;

    // 글 작성
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::post_json(
                "/posts",
                json!({"title": "스냅샷 테스트 글", "content": "이 응답 구조를 고정합니다"}),
            ),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let json = common::response_json(response).await;

    assert_json_snapshot!(json, {".id" => "[id]", ".author_id" => "[author_id]", ".created_at" => "[created_at]", ".updated_at" => "[updated_at]"});
}

#[tokio::test]
async fn snapshot_unauthorized_error() {
    let ctx = common::TestContext::new();

    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/posts",
            json!({"title": "No Token", "content": "실패"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json = common::response_json(response).await;

    assert_json_snapshot!(json, {".error" => "unauthorized", ".message" => "[message]"});
}

#[tokio::test]
async fn snapshot_not_found_error() {
    let ctx = common::TestContext::new();

    let response = ctx
        .app()
        .oneshot(common::get("/posts/999999"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = common::response_json(response).await;

    assert_json_snapshot!(json, {".error" => "not_found", ".message" => "[message]"});
}

#[tokio::test]
async fn snapshot_list_posts() {
    let ctx = common::TestContext::new();
    let token = ctx.signup_and_login().await;

    // 글 2개 작성
    for title in ["첫 번째 글", "두 번째 글"] {
        ctx.app()
            .oneshot(common::with_token(
                common::post_json("/posts", json!({"title": title, "content": "본문"})),
                &token,
            ))
            .await
            .unwrap();
    }

    let response = ctx.app().oneshot(common::get("/posts")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = common::response_json(response).await;
    let json_posts = json["posts"].clone();

    assert_json_snapshot!(
        json_posts,
        {
            "[].id" => "[id]",
            "[].author_id" => "[author_id]",
            "[].created_at" => "[timestamp]",
            "[].updated_at" => "[timestamp]",
        }
    );
}
