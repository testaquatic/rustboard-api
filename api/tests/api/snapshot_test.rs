use axum::http::StatusCode;
use insta::assert_json_snapshot;
use rustboard_api::test_utils::{helper, test_server::TestServer};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn snapshot_create_post_response() {
    let test_server = TestServer::new_in_memory().await;

    // 로그인
    let token = test_server
        .create_test_token("test@example.com", "test1234", "Tester")
        .await;

    // 글 작성
    let response = test_server
        .app_router
        .oneshot(helper::with_token(
            helper::post_json(
                "/posts",
                json!({"title": "스냅샷 테스트 글", "content": "이 응답 구조를 고정합니다"}),
            ),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let json = helper::response_json(response).await;

    assert_json_snapshot!(json, {".id" => "[id]", ".author_id" => "[author_id]", ".created_at" => "[created_at]", ".updated_at" => "[updated_at]"});
}

#[tokio::test]
async fn snapshot_unauthorized_error() {
    let test_server = TestServer::new_in_memory().await;

    let response = test_server
        .app_router
        .oneshot(helper::post_json(
            "/posts",
            json!({"title": "No Token", "content": "실패"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json = helper::response_json(response).await;

    assert_json_snapshot!(json, {".error" => "unauthorized", ".message" => "[message]"});
}

#[tokio::test]
async fn snapshot_not_found_error() {
    let test_server = TestServer::new_in_memory().await;

    let response = test_server
        .app_router
        .oneshot(helper::get("/posts/999999"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = helper::response_json(response).await;

    assert_json_snapshot!(json, {".error" => "not_found", ".message" => "[message]"});
}

#[tokio::test]
async fn snapshot_list_posts() {
    let test_server = TestServer::new_in_memory().await;
    let token = test_server
        .create_test_token("test@example.com", "test1234", "Tester")
        .await;

    // 글 2개 작성
    for title in ["첫 번째 글", "두 번째 글"] {
        test_server
            .app_router
            .clone()
            .oneshot(helper::with_token(
                helper::post_json("/posts", json!({"title": title, "content": "본문"})),
                &token,
            ))
            .await
            .unwrap();
    }

    let response = test_server
        .app_router
        .clone()
        .oneshot(helper::get("/posts"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = helper::response_json(response).await;
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
