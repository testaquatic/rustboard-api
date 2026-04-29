use std::sync::Arc;

use rustboard_api::{
    config::Config, repository::post::InMemoryPostRepository, router::app_routes,
    service::post::PostService, state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env 파일이 있으면 로드한다.
    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env()?);

    // 리포지토리 초기화
    let repo = Arc::new(InMemoryPostRepository::new());

    // 서비스 초기화
    let posts_service = Arc::new(PostService::new(repo));

    // AppState 생성
    let state = AppState {
        config: config.clone(),
        posts_service,
    };

    // 라우터를 생성하고 상태 붙이기
    let app = app_routes(&config).with_state(state);

    // 서버 시작
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    println!(
        "{} listening on http://{}",
        config.service_name,
        listener.local_addr()?
    );

    axum::serve(listener, app).await?;

    Ok(())
}
