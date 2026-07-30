use std::sync::Arc;

use rustboard_api::{
    repository::post::InMemoryPostRepository, router::app_routes, service::post::PostService,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 리포지토리 초기화
    let repo = Arc::new(InMemoryPostRepository::new());

    // 서비스에 리포지토리 주입
    let post_service = Arc::new(PostService::new(repo));

    // AppState에 담기
    let state = AppState { post_service };

    // 라우터를 만들고 상태 붙이기
    let app = app_routes().with_state(state);

    // 서버 실행
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!(
        "rustboard-api listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}
