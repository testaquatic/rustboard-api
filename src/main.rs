use axum::{Json, Router, routing::get};
use serde::Serialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "rustboard-api",
    })
}

#[derive(Serialize)]
pub struct VersionResponse {
    service: &'static str,
    version: &'static str,
}

async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        service: "rustboard-api",
        version: VERSION,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/version", get(version));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!(
        "rustboard-api listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}
