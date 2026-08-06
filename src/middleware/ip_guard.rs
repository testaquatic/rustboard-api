use std::{net::SocketAddr, pin::Pin};

use axum::{
    Json,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tower::{Layer, Service};

const ALLOWED_IPS: &[&str] = &["127.0.0.1", "::1"];

#[derive(Clone)]
pub struct IpGuardLayer;

impl<S> Layer<S> for IpGuardLayer {
    type Service = IpGuardService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        IpGuardService { inner }
    }
}

#[derive(Clone)]
pub struct IpGuardService<S> {
    inner: S,
}

impl<S> Service<Request> for IpGuardService<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, req: Request) -> Self::Future {
        let Some(ip_addr) = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip().to_string())
        else {
            return Box::pin(async {
                Ok((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "forbidden",
                        "message": "IP를 확인할 수 없습니다.",
                    })),
                )
                    .into_response())
            });
        };

        let future = self.inner.call(req);

        Box::pin(async move {
            if !ALLOWED_IPS.contains(&ip_addr.as_str()) {
                tracing::warn!(client_ip = %ip_addr, "허용되지 않은 IP에서 관리 엔드포인트 접근 시도");
                return Ok((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "forbidden",
                        "message": "접근이 허용되지 않은 IP입니다."
                    })),
                )
                    .into_response());
            }

            let response = future.await?;
            Ok(response)
        })
    }
}
