use rustboard_api::{
    configuration::get_configuration,
    startup::{self},
    telemetry,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 설정을 읽는다
    let configuration = get_configuration()?;

    // 로깅
    let _guard = telemetry::init_telemetry(&configuration)?;

    // DB 풀 만들기
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&configuration.database.database_url())
        .await?;

    // 서버 실행
    let listener = tokio::net::TcpListener::bind(configuration.bind_addr).await?;
    tracing::info!(
        "{} listening on http://{}",
        configuration.service_name,
        listener.local_addr()?
    );

    startup::run_app(listener, pool.clone(), configuration.into()).await?;

    Ok(())
}
