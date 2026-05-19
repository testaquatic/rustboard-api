use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn signup_duplicate_email_returns_422() {
    let ctx = common::InMemoryTestContext::new_in_memory();

    // 회원가입
    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/signup",
            json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    // 중복 회원가입
    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/signup",
            // 이메일 주소만 같다.
            json!({
                "email": "test@example.com",
                "password": "password12345",
                "display_name": "Tester2",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
}
