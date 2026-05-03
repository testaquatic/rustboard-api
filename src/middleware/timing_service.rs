use std::{pin::Pin, time::Instant};

use axum::{extract::Request, response::Response};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TimingLayer;

impl<S> Layer<S> for TimingLayer {
    type Service = TimingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimingService { inner }
    }
}

#[derive(Clone)]
pub struct TimingService<S> {
    inner: S,
}

impl<S> Service<Request> for TimingService<S>
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
        let method = req.method().clone();
        let uri = req.uri().clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let start = Instant::now();
            let response = inner.call(req).await?;
            let elapsed = start.elapsed();

            tracing::info!(
              method = %method,
              uri = %uri,
              status = %response.status().as_u16(),
              elapsed = %elapsed.as_millis(),
              "요청 처리 완료 (Service 구현)"
            );

            Ok(response)
        })
    }
}
