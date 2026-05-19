use std::{net::SocketAddr, pin::Pin};

use axum::{
    Json,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tower::{Layer, Service};

const ALLOWED_IPS: &[&str] = &["127.0.0.1", "::1"];

pub async fn require_allowed_ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let client_ip = addr.ip().to_string();

    if ALLOWED_IPS.iter().any(|&allowed| allowed == client_ip) {
        next.run(req).await
    } else {
        tracing::warn!(
          client_ip = %client_ip,
          "허용되지 않은 IP에서 관리 엔드포인트 접근 시도",
        );
        (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden", "message": "접근이 허용되지 않은 IP입니다."})),
        )
            .into_response()
    }
}

#[derive(Clone)]
pub struct AllowedIPLayer;

impl<S> Layer<S> for AllowedIPLayer {
    type Service = AllowedIPMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service {
        AllowedIPMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct AllowedIPMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for AllowedIPMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, req: Request) -> Self::Future {
        let ConnectInfo(addr) = req.extensions().get::<ConnectInfo<SocketAddr>>().unwrap();
        let client_ip = addr.ip().to_string();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if ALLOWED_IPS.iter().any(|&allowed| allowed == client_ip) {
                inner.call(req).await
            } else {
                tracing::warn!(
                  client_ip = %client_ip,
                  "허용되지 않은 IP에서 관리 엔드포인트 접근 시도",
                );
                let response = (
                    StatusCode::FORBIDDEN,
                    Json(
                        json!({"error": "forbidden", "message": "접근이 허용되지 않은 IP입니다."}),
                    ),
                )
                    .into_response();

                Ok(response)
            }
        })
    }
}
