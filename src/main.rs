use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::http::StatusCode;
use rustboard_api::{
    configuration::Settings, repository::post::PostsRepository, routes::app_router,
    service::post::PostsService, state::AppState, telemetry::init_telemetry,
};
use sqlx::postgres::PgPoolOptions;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

#[tokio::main]
async fn main() {
    // 설정을 불러온다.
    let settings = Settings::build().expect("Failed to build settings");

    // 로깅을 시작한다.
    let _otel_guard = init_telemetry(&settings).expect("Failed to initialize telemetry");

    // PgPool을 생성한다.
    let pool = PgPoolOptions::new()
        .connect_lazy(&settings.database_url)
        .expect("Failed to connect to database");

    // 애플리케이션 상태를 생성한다.
    let state = AppState {
        configuration: Arc::new(settings),
        posts_service: Arc::new(PostsService::new(PostsRepository::new(pool))),
    };

    // Listener를 생성한다.
    let listener = tokio::net::TcpListener::bind(&state.configuration.app_addr)
        .await
        .expect("Failed to bind to address");

    // Rate limit 설정 (15분당 100회)
    let governor_config = GovernorConfigBuilder::default()
        .per_second(15 * 60)
        .burst_size(100)
        .finish()
        .expect("Failed to create governor config");

    // Axum 라우터 설정
    let app = app_router()
        .layer(GovernorLayer::new(governor_config))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Tower의 governor는 request의 connect_info를 사용하기 위해 SocketAddr를 필요로 한다.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to serve")
}
