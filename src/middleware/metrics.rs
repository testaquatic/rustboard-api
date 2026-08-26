use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

pub async fn track_metrics(req: Request, next: Next) -> Response {
    let meter = opentelemetry::global::meter("rustboard_api");
    let request_counter = meter
        .u64_counter("http.server.requeest.count")
        .with_description("HTTP 요청 총 수")
        .build();
    let latency_histogram = meter
        .f64_histogram("http.server.requeest.duration")
        .with_description("HTTP 요청 처리 시간 (초)")
        .with_unit("s")
        .build();

    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();

    let attributes = [
        opentelemetry::KeyValue::new("http.method", method),
        opentelemetry::KeyValue::new("http.route", path),
        opentelemetry::KeyValue::new("http.status_code", status),
    ];

    request_counter.add(1, &attributes);
    latency_histogram.record(duration, &attributes);

    response
}
