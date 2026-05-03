use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

pub async fn add_request_id(mut req: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();

    req.headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());

    let span = tracing::info_span!("request", request_id = %request_id);
    let _guard = span.enter();

    let mut response = next.run(req).await;

    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());

    response
}
