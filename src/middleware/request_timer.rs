use axum::{extract::Request, middleware::Next, response::Response};
use tokio::time::Instant;

pub async fn reqeust_timer(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    tracing::info!("{} {} - {:?}", method, uri, duration);

    response
}
