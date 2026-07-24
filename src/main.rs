use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/", get(hello));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!(
        "rustboard-api listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello() -> &'static str {
    "Hello, Axum!"
}
