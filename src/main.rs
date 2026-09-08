use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{http::StatusCode, middleware};
use rustboard_api::{
    configuration::get_configuration,
    middleware::{
        ip_guard::IpGuardLayer, metrics::track_metrics,
        rate_limit_error::rate_limit_error_response, rate_limit_key::ForwardedIpKeyExtractor,
        request_id::AddRequestIdLayer,
    },
    repository::{comment::CommentRepository, post::PostRepository, user::UserRepository},
    router::create_router,
    service::{comment::CommentService, post::PostService, user::UserService},
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
    // 설정을 읽는다
    let configuration = Arc::new(get_configuration()?);

    // 로깅
    let _guard = telemetry::init_telemetry(&configuration)?;

    // DB 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.database_url())
        .await?;

    // 앱 부팅 시 마이그레이션 자동 적용
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 리포지토리 초기화
    let posts_repo = PostRepository::new(pool.clone());
    let comments_repo = CommentRepository::new(pool.clone());
    let users_repo = UserRepository::new(pool.clone());

    // 서비스에 리포지토리 주입
    let post_service = Arc::new(PostService::new(posts_repo.clone()));
    let comment_service = Arc::new(CommentService::new(posts_repo, comments_repo));
    let user_service = Arc::new(UserService::new(users_repo));

    // AppState에 담기
    let state = AppState {
        post_service,
        comment_service,
        configuration: configuration.clone(),
        pool: pool.clone(),
        user_service,
    };

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(30)
        .key_extractor(ForwardedIpKeyExtractor)
        .finish()
        .unwrap();

    let governor_layer = GovernorLayer::new(governor_conf).error_handler(rate_limit_error_response);

    let app = create_router(state)
        // 모든 라우트에 적용
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(governor_layer)
        .layer(IpGuardLayer)
        .layer(middleware::from_fn(track_metrics))
        .layer(TraceLayer::new_for_http())
        .layer(AddRequestIdLayer);
    // 서버 실행
    let listener = tokio::net::TcpListener::bind(configuration.bind_addr).await?;
    tracing::info!(
        "{} listening on http://{}",
        configuration.service_name,
        listener.local_addr()?
    );
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        tracing::error!(error = %e, "서버 에러");
    }

    tracing::info!("리소스 정리를 시작합니다 (최대 10초)");
    tokio::select! {
        _ = cleanup(pool) => {
            tracing::info!("리소스 정리 완료");
        }

        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            tracing::warn!("리소스 정리 타임아웃, 강제 종료합니다");
        }
    }

    tracing::info!("서버 종료 완료");

    Ok(())
}

async fn cleanup(pool: sqlx::PgPool) {
    pool.close().await;
    tracing::info!("DB 커넥션 풀 종료");
}
