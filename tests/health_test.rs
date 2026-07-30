use std::sync::Arc;

use axum::http::Request;
use rustboard_api::{
    repository::post::InMemoryPostRepository, router::app_routes, service::post::PostService,
    state::AppState,
};
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_200_without_db() {
    let repo = Arc::new(InMemoryPostRepository::new());
    let state = AppState {
        post_service: Arc::new(PostService::new(repo)),
    };

    let app = app_routes().with_state(state);
    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}
