use std::convert::Infallible;

use axum::extract::FromRequestParts;

#[derive(Debug)]
pub struct RequestID(pub String);

impl<S> FromRequestParts<S> for RequestID
where
    S: Send + Sync,
{
    type Rejection = Infallible;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let id = parts
            .headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(RequestID(id))
    }
}
