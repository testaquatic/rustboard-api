use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use crate::common;

#[tokio::test]
async fn signup_duplicate_email_returns_422() {
    let ctx = common::TestContext::new().await;

    // 첫 회원가입
    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/signup",
            &json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 동일한 이메일로 다시 회원가입 시도
    let response = ctx
        .app()
        .oneshot(common::post_json(
            "/signup",
            &json!({
                "email": "test@example.com",
                "password": "password123",
                "display_name": "Tester2",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
