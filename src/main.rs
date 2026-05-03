use std::sync::Arc;

use rustboard_api::{
    config::Config,
    repository::{comment::PostgresCommentRepository, posts::PostgresPostRepository},
    router::app_routes,
    service::{comments::CommentService, posts::PostService},
    state::AppState,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env 파일이 있으면 로드한다.
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustboard_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 설정 읽기
    let config = Arc::new(Config::from_env()?);

    // DB 연결 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // 앱을 시작하면 DB마이그레이션을 자동 적용한다.
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 리포지토리 초기화
    let posts_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));

    // 서비스 초기화
    let posts_service = Arc::new(PostService::new(posts_repo.clone()));
    let comments_service = Arc::new(CommentService::new(posts_repo, comments_repo));

    // AppState 생성
    let state = AppState {
        config: config.clone(),
        pool,
        posts_service,
        comments_service,
    };

    // 라우터를 생성하고 상태 붙이기
    let app = app_routes(&config)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(
            rustboard_api::middleware::request_id::add_request_id,
        ));

    // 서버 시작
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(
        "{} listening on http://{}",
        config.service_name,
        listener.local_addr()?
    );

    axum::serve(listener, app).await?;

    Ok(())
}
