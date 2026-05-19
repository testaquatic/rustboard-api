use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;

use crate::common;

#[tokio::test]
async fn health_check_returns_200_and_ok() {
    let app = common::InMemoryTestContext::new_in_memory().app();

    let response = app.oneshot(common::get("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(
        common::response_json(response).await,
        json!({"status": "ok", "service": "rustboard-api-test"})
    );
}
