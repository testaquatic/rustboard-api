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
    shutdown::shutdown_signal,
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

    // 라우터를 생성하고 상태와 미들웨어를 붙인다
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

    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        tracing::error!(error = %e, "서버 오류");
    }

    // 서버 종료 후 리소스 정리
    tracing::info!("리소스 정리를 시작합니다 (최대 10초)");

    tokio::select! {
        _ = cleanup(pool.clone()) => tracing::info!("리소스 정리 완료"),
        _ = tokio::time::sleep(Duration::from_secs(10)) => tracing::warn!("리소스 정리 타입아웃, 강제 종료합니다"),
    }

    tracing::info!("서버 종료 완료");

    Ok(())
}

async fn cleanup(pool: sqlx::PgPool) {
    // DB 커넥션 풀 정리
    pool.close().await;
    tracing::info!("DB 커넥션 풀 정리 완료");
}
