use std::sync::Arc;

use rustboard_api::{
    configuration::get_configuration, repository::post::InMemoryPostRepository, router::app_routes,
    service::post::PostService, state::AppState, swagger::get_swagger_router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 설정을 읽는다
    let configuration = Arc::new(get_configuration()?);

    // 리포지토리 초기화
    let repo = Arc::new(InMemoryPostRepository::new());

    // 서비스에 리포지토리 주입
    let post_service = Arc::new(PostService::new(repo));

    // AppState에 담기
    let state = AppState {
        post_service,
        configuration: configuration.clone(),
    };

    // 라우터를 만들고 상태 붙이기
    let app = app_routes()
        .with_state(state.clone())
        .merge(get_swagger_router(state));

    // 서버 실행
    let listener = tokio::net::TcpListener::bind(configuration.bind_addr).await?;
    println!(
        "{} listening on http://{}",
        configuration.service_name,
        listener.local_addr()?,
    );
    axum::serve(listener, app).await?;

    Ok(())
}
