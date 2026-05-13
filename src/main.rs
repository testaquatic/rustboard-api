use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::http::StatusCode;
use rustboard_api::{
    config::Config,
    middleware::{
        self, rate_limit_error::rete_limit_error_response, rate_limit_key::ForwardedIpKeyExtractor,
    },
    repository::{
        comment::PostgresCommentRepository, posts::PostgresPostRepository,
        user::PostgresUserRepository,
    },
    router::create_router,
    service::{comments::CommentService, posts::PostService, user::UserService},
    state::AppState,
    telemetry,
};
use sqlx::postgres::PgPoolOptions;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // env 파일이 있으면 로드한다.
    dotenvy::dotenv().ok();

    // 로그 설정
    // 우아한 종료를 위한 _gurad
    let _guard = telemetry::init_telemetry()?;

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
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let users_service = Arc::new(UserService::new(user_repo));

    // AppState 생성
    let state = AppState {
        config: config.clone(),
        posts_service,
        comments_service,
        users_service,
    };

    // 동시 접속수를 제한하는 레이어
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .key_extractor(ForwardedIpKeyExtractor)
        .finish()
        .unwrap();
    let governor_layer = GovernorLayer::new(governor_conf).error_handler(rete_limit_error_response);

    // 라우터를 생성하고 상태 붙이기
    let app = create_router(&config, state.clone())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(governor_layer)
        .layer(middleware::metric::TrackMetricsLayer)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(
            middleware::request_id::add_request_id,
        ));

    // 서버 시작
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(
        "{} listening on http://{}",
        config.service_name,
        listener.local_addr()?
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM 시그널 핸들러 설치 실패")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {tracing::info!("Ctrl+C 수신, 종료 시작")},
        _ = terminate => {tracing::info!("SIGTERM 수신, 종료 시작")},
    }
}
