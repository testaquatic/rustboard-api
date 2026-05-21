use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use crate::common::{self, server::TestServer};

#[tokio::test]
async fn full_auth_flow() {
    // Postgres 준비
    let test_server = TestServer::new_in_memory().await;

    // 회원 가입 - Alice
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::post_json(
            "/signup",
            json!({
              "email": "alice@test.com",
              "password": "pass1234",
              "display_name": "Alice"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 회원 가입 - Bob
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::post_json(
            "/signup",
            json!({
              "email": "bob@test.com",
              "password": "pass5678",
              "display_name": "Bob"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 로그인 - Alice
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::post_json(
            "/login",
            json!({"email": "alice@test.com", "password": "pass1234"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = common::helper::response_json(response).await;
    let alice_token = json["token"].as_str().unwrap().to_string();

    // 로그인 - Bob
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::post_json(
            "/login",
            json!({"email": "bob@test.com", "password": "pass5678"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = common::helper::response_json(response).await;
    let bob_token = json["token"].as_str().unwrap().to_string();

    // 비인증 글 작성 시도
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::post_json(
            "/posts",
            json!({"title": "No Token", "content": "실패해야 함"}),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 글 작성 - Alice
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::with_token(
            common::helper::post_json(
                "/posts",
                json!({"title": "Alice의 글", "content": "안녕하세요"}),
            ),
            &alice_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let json = common::helper::response_json(response).await;
    let post_id = json["id"].as_i64().unwrap();

    // 비 인증 글 조회
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Bob이 Alice의 글을 수정 시도
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::with_token(
            common::helper::patch_json(
                &format!("/posts/{}", post_id),
                json!({"title": "Bob이 수정", "content": "실패해야 함"}),
            ),
            &bob_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Alice가 자기 글 수정
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::with_token(
            common::helper::patch_json(
                &format!("/posts/{}", post_id),
                json!({"title": "수정됨", "content": "수정된 내용"}),
            ),
            &alice_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Alice가 자기 글 삭제
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::with_token(
            common::helper::delete(&format!("/posts/{}", post_id)),
            &alice_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제된 글 조회
    let app = test_server.app_router.clone();
    let response = app
        .oneshot(common::helper::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
