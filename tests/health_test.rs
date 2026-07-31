use std::sync::Arc;

use axum::http::Request;
use rustboard_api::{
    configuration::get_configuration, repository::post::InMemoryPostRepository, router::app_routes,
    service::post::PostService, state::AppState,
};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_without_db() {
    let repo = Arc::new(InMemoryPostRepository::new());
    let configuration = Arc::new(get_configuration().expect("Failed to get configuration"));
    let state = AppState {
        post_service: Arc::new(PostService::new(repo)),
        configuration,
    };

    let app = app_routes().with_state(state);
    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
