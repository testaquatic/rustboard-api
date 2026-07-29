use axum::{Json, Router, extract::State, routing::get};
use rustboard_api::state::AppState;
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

async fn version(State(state): State<AppState>) -> Json<VersionResponse> {
    Json(VersionResponse {
        service: state.service_name,
        version: VERSION,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState {
        service_name: "rustboard-api",
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!(
        "rustboard-api listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}
