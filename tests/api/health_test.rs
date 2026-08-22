use axum::http::StatusCode;
use tower::ServiceExt;

use crate::common::{self, TestContext};

#[tokio::test]
async fn health_check_returns_200_and_ok() {
    let app = TestContext::new().await.app();

    let response = app.oneshot(common::get("/health")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_json = common::response_json(response).await;
    assert_eq!(response_json["status"].as_str().unwrap(), "ok");
}
