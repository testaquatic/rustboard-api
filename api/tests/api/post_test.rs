use axum::http::StatusCode;
use rustboard_domain::posts::Post;
use serde_json::json;
use tower::ServiceExt;

use crate::common::{self, server::TestServer};

#[tokio::test]
async fn create_post_without_token_returns_401() {
    let request = common::helper::post_json(
        "/posts",
        serde_json::json!({"title": "테스트 글", "content": "본문입니다"}),
    );
    let response = TestServer::new_in_memory()
        .await
        .app_router
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nonexistent_post_returns_404() {
    let response = TestServer::new_in_memory()
        .await
        .app_router
        .oneshot(common::helper::get("/posts/999999"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn signup_then_login_then_create_post() {
    let test_server = TestServer::new_in_memory().await;

    // 로그인
    let token = test_server
        .create_test_token("test@example.com", "test1234", "Tester")
        .await;

    // 글 작성
    let response = test_server
        .app_router
        .oneshot(common::helper::with_token(
            common::helper::post_json(
                "/posts",
                json!({"title": "Alice의 첫 글", "content": "테스트입니다"}),
            ),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn list_returns_empty_then_no_posts() {
    let response = TestServer::new_in_memory()
        .await
        .app_router
        .oneshot(common::helper::get("/posts"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = common::helper::response_json(response).await;
    assert!(json["posts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_returns_seeded_posts() {
    let test_server = TestServer::new_in_memory().await;

    // 사전데이터 주입
    let posts = vec![
        Post {
            id: 1,
            title: "첫 번째".to_string(),
            body: "내용 1".to_string(),
            author_id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Post {
            id: 2,
            title: "두 번째".to_string(),
            body: "내용 2".to_string(),
            author_id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    let token = test_server
        .create_test_token("tester@example.com", "test1234", "Tester")
        .await;

    for post in posts {
        test_server.create_post(&token, &post).await;
    }

    let response = test_server
        .app_router
        .oneshot(common::helper::get("/posts"))
        .await
        .unwrap();
    let json = common::helper::response_json(response).await;

    assert_eq!(json["posts"].as_array().unwrap().len(), 2);
    assert_eq!(json["posts"][0]["title"], "첫 번째");
    assert_eq!(json["posts"][1]["title"], "두 번째");
}

#[tokio::test]
async fn owner_can_delete_own_post() {
    let test_server = TestServer::new_in_memory().await;

    // 계정 생성과 로그인
    let token = test_server
        .create_test_token("tester@example.com", "test1234", "Tester")
        .await;

    // 글 작성
    let response = test_server
        .app_router
        .clone()
        .oneshot(common::helper::with_token(
            common::helper::post_json(
                "/posts",
                json!({"title": "삭제 테스트", "content": "곧 지워질 글"}),
            ),
            &token,
        ))
        .await
        .unwrap();
    let json = common::helper::response_json(response).await;
    let post_id = json["id"].as_i64().unwrap();

    // 삭제
    let response = test_server
        .app_router
        .clone()
        .oneshot(common::helper::with_token(
            common::helper::delete(&format!("/posts/{}", post_id)),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제 확인
    let response = test_server
        .app_router
        .oneshot(common::helper::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
