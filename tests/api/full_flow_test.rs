use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn full_auth_flow() {
    // 테스트 환경 구성
    let test_context = common::TestContext::new().await;

    // 회원 가입 - Alice
    let response = test_context
        .app()
        .oneshot(common::post_json(
            "/signup",
            &json!(
                {
                    "email": "alice@example.com",
                    "password": "pass1234",
                    "display_name": "Alice",
                }
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 회원가입 - Bob
    let response = test_context
        .app()
        .oneshot(common::post_json(
            "/signup",
            &json!({
                "email": "bob@example.com",
                "password": "pass5678",
                "display_name": "Bob",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 로그인 - Alice
    let response = test_context
        .app()
        .oneshot(common::post_json(
            "/login",
            &json!({
                "email": "alice@example.com",
                "password": "pass1234",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = common::response_json(response).await;
    let alice_token = json["token"].as_str().unwrap();

    // 로그인 - Bob
    let response = test_context
        .app()
        .oneshot(common::post_json(
            "/login",
            &json!({
                "email": "bob@example.com",
                "password": "pass5678",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = common::response_json(response).await;
    let bob_token = json["token"].as_str().unwrap();

    // 비인증 글 작성 시도
    let response = test_context
        .app()
        .oneshot(common::post_json(
            "/posts",
            &json!({ "title": "No Token", "content": "실패해야 함" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // alice가 글 작성
    let response = test_context
        .app()
        .oneshot(common::with_token(
            common::post_json(
                "/posts",
                &json!({ "title": "Alice의 글", "content": "안녕하세요" }),
            ),
            alice_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let json = common::response_json(response).await;
    let post_id = json["id"].as_i64().unwrap();

    // 비인증 글 조회
    let response = test_context
        .app()
        .oneshot(common::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Bob이 Alice의 글 수정 시도
    let response = test_context
        .app()
        .oneshot(common::with_token(
            common::patch_json(
                &format!("/posts/{}", post_id),
                &json!({"title": "Bob이 수정", "content": "실패해야 함"}),
            ),
            bob_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Alice가 자기 글 수정
    let response = test_context
        .app()
        .oneshot(common::with_token(
            common::patch_json(
                &format!("/posts/{}", post_id),
                &json!({ "title": "Alice가 수정", "content": "성공" }),
            ),
            alice_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Bob이 Alice 글 삭제 시도
    let response = test_context
        .app()
        .oneshot(common::with_token(
            common::delete(&format!("/posts/{}", post_id)),
            bob_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Alice가 자기 글 삭제
    let response = test_context
        .app()
        .oneshot(common::with_token(
            common::delete(&format!("/posts/{}", post_id)),
            alice_token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제된 글 조회
    let response = test_context
        .app()
        .oneshot(common::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
