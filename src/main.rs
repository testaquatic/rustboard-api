mod extract;

use axum::{Router, extract::Path, routing::get};

use crate::extract::RequestID;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(hello))
        .route("/posts/{id}", get(get_post))
        .route("/whoami", get(whoami));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
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

async fn get_post(Path(id): Path<i64>) -> String {
    format!("post id = {id}")
}

async fn whoami(request_id: RequestID) -> String {
    format!("your request id is {}", request_id.0)
}
