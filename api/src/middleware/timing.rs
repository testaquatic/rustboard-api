use axum::{extract::Request, middleware::Next, response::Response};

pub async fn measure_request_time(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let start_time = std::time::Instant::now();

    let response = next.run(req).await;

    let elapsed = start_time.elapsed();

    tracing::info!(
      method = %method,
      url = %uri,
      status = %response.status().as_u16(),
      elapsed = %elapsed.as_millis(),
      "요청 처리 완료"
    );

    response
}
