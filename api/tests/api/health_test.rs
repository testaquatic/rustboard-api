use axum::http::StatusCode;
use serde_json::json;
use tower::util::ServiceExt;

use crate::common::{self, server::TestServer};

#[tokio::test]
async fn health_check_returns_200_and_ok() {
    let app = TestServer::new_in_memory().await.app_router.clone();

    let response = app.oneshot(common::helper::get("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(
        common::helper::response_json(response).await,
        json!({"status": "ok", "service": "rustboard-api-test"})
    );
}
