use axum::{Router, routing::get};
mod extract;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/whoami", get(whoami));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!(
        "rustboard-api listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}

async fn whoami(extract::RequestId(id): extract::RequestId) -> String {
    format!("your request id is {}", id)
}
