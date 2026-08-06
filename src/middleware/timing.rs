use axum::{extract::Request, response::Response};
use std::pin::Pin;
use std::time::Instant;
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
        let method = req.method().clone();
        let uri = req.uri().clone();
        let start = Instant::now();

        let future = self.inner.call(req);

        Box::pin(async move {
            let response = future.await?;

            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_millis();

            tracing::info!(
                method = %method,
                uri = %uri,
                status = %response.status(),
                elapsed_ms = %elapsed_ms,
                "요청 처리 완료"
            );

            Ok(response)
        })
    }
}
