use std::{pin::Pin, time::Instant};

use axum::{extract::Request, response::Response};
use opentelemetry_semantic_conventions::attribute;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TrackMetricsLayer;

impl<S> Layer<S> for TrackMetricsLayer {
    type Service = TrackMetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TrackMetricsMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct TrackMetricsMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for TrackMetricsMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
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
        let meter = opentelemetry::global::meter("rustboard-api");
        let request_counter = meter
            .u64_counter("http.server.request.count")
            .with_description("HTTP 요청 총 수")
            .build();
        let latency_histogram = meter
            .f64_histogram("http.server.request.duration")
            .with_description("HTTP 요청 처리 시간 (초)")
            .with_unit("s")
            .build();

        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let start = Instant::now();

        let response_future = self.inner.call(req);

        Box::pin(async move {
            let response = response_future.await?;
            let duration = start.elapsed().as_secs_f64();
            let status = response.status().as_u16().to_string();

            let attributes = [
                opentelemetry::KeyValue::new("http.method", method),
                opentelemetry::KeyValue::new("http.status", status),
                opentelemetry::KeyValue::new(attribute::HTTP_ROUTE, path),
            ];

            request_counter.add(1, &attributes);
            latency_histogram.record(duration, &attributes);

            Ok(response)
        })
    }
}
