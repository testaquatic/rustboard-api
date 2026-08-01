use std::sync::Arc;

use rustboard_api::{
    configuration::get_configuration, repository::post::PostgresPostRepository, router::app_routes,
    service::post::PostService, state::AppState, swagger::get_swagger_router,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 설정을 읽는다
    let configuration = Arc::new(get_configuration()?);

    // DB 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database_url)
        .await?;

    // 앱 부팅 시 마이그레이션 자동 적용
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 리포지토리 초기화
    let repo = Arc::new(PostgresPostRepository::new(pool.clone()));

    // 서비스에 리포지토리 주입
    let post_service = Arc::new(PostService::new(repo));

    // AppState에 담기
    let state = AppState {
        post_service,
        configuration: configuration.clone(),
        pool,
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
