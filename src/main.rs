use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::http::StatusCode;
use rustboard_api::{
    configuration::get_configuration,
    middleware::{
        ip_guard::IpGuardLayer, rate_limit_error::rate_limit_error_response,
        rate_limit_key::ForwardedIpKeyExtractor, request_id::AddRequestIdLayer,
        timing::TimingLayer,
    },
    repository::{comment::PostgresCommentRepository, post::PostgresPostRepository},
    router::app_routes,
    service::{comment::CommentService, post::PostService},
    state::AppState,
    swagger::get_swagger_router,
};
use sqlx::postgres::PgPoolOptions;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 설정을 읽는다
    let configuration = Arc::new(get_configuration()?);

    // 로깅
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustboard_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // DB 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database_url)
        .await?;

    // 앱 부팅 시 마이그레이션 자동 적용
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 리포지토리 초기화
    let posts_repo = Arc::new(PostgresPostRepository::new(pool.clone()));
    let comments_repo = Arc::new(PostgresCommentRepository::new(pool.clone()));

    // 서비스에 리포지토리 주입
    let post_service = Arc::new(PostService::new(posts_repo.clone()));
    let comment_service = Arc::new(CommentService::new(posts_repo, comments_repo));

    // AppState에 담기
    let state = AppState {
        post_service,
        comment_service,
        configuration: configuration.clone(),
        pool,
    };

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .key_extractor(ForwardedIpKeyExtractor)
        .finish()
        .unwrap();

    let governor_layer = GovernorLayer::new(governor_conf).error_handler(rate_limit_error_response);

    // 라우터를 만들고 상태 붙이기
    let app = app_routes()
        .with_state(state.clone())
        .merge(get_swagger_router(state))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(governor_layer)
        .layer(IpGuardLayer)
        .layer(TimingLayer)
        .layer(TraceLayer::new_for_http())
        .layer(AddRequestIdLayer);

    // 서버 실행
    let listener = tokio::net::TcpListener::bind(configuration.bind_addr).await?;
    tracing::info!(
        "{} listening on http://{}",
        configuration.service_name,
        listener.local_addr()?
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
