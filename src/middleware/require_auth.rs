use axum::{extract::Request, middleware::Next};

pub async fn require_auth(req: Request, next: Next) {
    next.run(req).await;
}
