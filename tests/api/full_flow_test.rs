use axum::http::StatusCode;
use serde_json::json;

use crate::common::test_context::{TestContext, response_json};

#[tokio::test]
async fn full_auth_flow() {
    // 테스트 환경 구성
    let test_context = TestContext::new().await;

    // 회원 가입 - Alice
    let response = test_context
        .post_json(
            "/signup",
            None,
            &json!(
                {
                    "email": "alice@example.com",
                    "password": "pass1234",
                    "display_name": "Alice",
                }
            ),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // 회원가입 - Bob
    let response = test_context
        .post_json(
            "/signup",
            None,
            &json!({
                "email": "bob@example.com",
                "password": "pass5678",
                "display_name": "Bob",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // 로그인 - Alice
    let response = test_context
        .post_json(
            "/login",
            None,
            &json!({
                "email": "alice@example.com",
                "password": "pass1234",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let json_body = response_json(response).await;
    let alice_token = json_body["token"].as_str().unwrap();

    // 로그인 - Bob
    let response = test_context
        .post_json(
            "/login",
            None,
            &json!({
                "email": "bob@example.com",
                "password": "pass5678",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let json_body = response_json(response).await;
    let bob_token = json_body["token"].as_str().unwrap();

    // 비인증 글 작성 시도
    let response = test_context
        .post_json(
            "/posts",
            None,
            &json!({ "title": "No Token", "content": "실패해야 함" }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // alice가 글 작성
    let response = test_context
        .post_json(
            "/posts",
            Some(alice_token),
            &json!({ "title": "Alice의 글", "content": "안녕하세요" }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let json_body = response_json(response).await;
    let post_id = json_body["id"].as_i64().unwrap();

    // 비인증 글 조회
    let (status_code, _) = test_context.get(&format!("/posts/{}", post_id), None).await;
    assert_eq!(status_code, StatusCode::OK);

    // Bob이 Alice의 글 수정 시도
    let (status_code, _) = test_context
        .patch_json(
            &format!("/posts/{}", post_id),
            Some(bob_token),
            &json!({"title": "Bob이 수정", "content": "실패해야 함"}),
        )
        .await;
    assert_eq!(status_code, StatusCode::FORBIDDEN);

    // Alice가 자기 글 수정
    let (status_code, _) = test_context
        .patch_json(
            &format!("/posts/{}", post_id),
            Some(alice_token),
            &json!({ "title": "Alice가 수정", "content": "성공" }),
        )
        .await;
    assert_eq!(status_code, StatusCode::OK);

    // Bob이 Alice 글 삭제 시도
    let response = test_context
        .delete(&format!("/posts/{}", post_id), Some(bob_token))
        .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Alice가 자기 글 삭제
    let response = test_context
        .delete(&format!("/posts/{}", post_id), Some(alice_token))
        .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제된 글 조회
    let (status_code, _) = test_context.get(&format!("/posts/{}", post_id), None).await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_signup_duplicate_email() {
    let test_context = TestContext::new().await;

    // 회원 가입 - Tester
    let response = test_context
        .post_json(
            "/signup",
            None,
            &json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // 동일한 이메일로 다시 회원가입 시도
    let response = test_context
        .post_json(
            "/signup",
            None,
            &json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester2",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let test_context = TestContext::new().await;

    // 회원 가입
    test_context
        .post_json(
            "/signup",
            None,
            &json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester",
            }),
        )
        .await;

    // 잘못된 비밀번호로 로그인 시도
    let response = test_context
        .post_json(
            "/login",
            None,
            &json!({
                "email": "test@example.com",
                "password": "wrong-password",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // 존재하지 않는 이메일로 로그인 시도
    let response = test_context
        .post_json(
            "/login",
            None,
            &json!({
                "email": "nonexistent@example.com",
                "password": "password123",
            }),
        )
        .await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
