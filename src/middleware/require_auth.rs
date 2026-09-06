use axum::{extract::Request, middleware::Next, response::Response};

pub async fn require_auth(req: Request, next: Next) -> Response {
    next.run(req).await
}
