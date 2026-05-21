use axum::http::StatusCode;
use insta::assert_json_snapshot;
use rustboard_api::test_utils::{helper, test_server::TestServer};
use tower::util::ServiceExt;

#[tokio::test]
async fn health_check_returns_200_and_ok() {
    let app = TestServer::new_in_memory().await.app_router.clone();

    let response = app.oneshot(helper::get("/health")).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = helper::response_json(response).await;

    assert_json_snapshot!(json, {".status" => "ok", ".service" => "[service]"});
}
