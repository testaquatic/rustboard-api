use axum::{extract::Request, middleware::Next, response::Response};

use crate::auth::extractor::AuthUser;

pub async fn require_auth(auth_user: AuthUser, req: Request, next: Next) -> Response {
    let mut req = req;
    req.extensions_mut().insert(auth_user);
    next.run(req).await
}
