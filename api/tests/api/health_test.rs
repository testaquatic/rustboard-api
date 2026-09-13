use axum::http::StatusCode;

use crate::common::test_context::TestContext;

#[tokio::test]
async fn health_check_returns_200_and_ok() {
    let test_context = TestContext::new().await;

    let (status_code, json_body) = test_context.get("/health", None).await;
    assert_eq!(status_code, StatusCode::OK);
    let response_json = json_body;
    assert_eq!(response_json["status"].as_str().unwrap(), "ok");
}
