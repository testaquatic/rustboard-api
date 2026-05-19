use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::instrument;

#[instrument("request", skip_all, fields(request_id = tracing::field::Empty))]
pub async fn add_request_id(mut req: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    tracing::Span::current().record("request_id", &request_id);

    req.headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());

    let mut response = next.run(req).await;

    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&request_id).unwrap());

    response
}
