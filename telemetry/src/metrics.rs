use opentelemetry::KeyValue;

/// DB 쿼리 지연 히스토그램
pub fn record_db_query_duration(operaton: &str, table: &str, duraction_secs: f64) {
    let meter = opentelemetry::global::meter("rustboard-api");
    let histogram = meter
        .f64_histogram("db.query.duration")
        .with_description("데이터베이스 쿼리 실행 시간 (초)")
        .with_unit("s")
        .build();

    histogram.record(
        duraction_secs,
        &[
            KeyValue::new("db.operation", operaton.to_string()),
            KeyValue::new("db.table", table.to_string()),
        ],
    );
}

/// 에러 카운터 (에러 종류별)
pub fn increment_error_counter(error_type: &str) {
    let meter = opentelemetry::global::meter("rustboard-api");
    let counter = meter
        .u64_counter("app.error.count")
        .with_description("애플리케이션 에러 발생 수")
        .build();

    counter.add(1, &[KeyValue::new("error.type", error_type.to_string())]);
}

/// 활성 DB 커넥션 수
pub fn update_active_connections(delta: i64) {
    let meter = opentelemetry::global::meter("rustboard-api");
    let counter = meter
        .i64_up_down_counter("db.connections.active")
        .with_description("현재 활성 DB 커넥션 수")
        .build();

    counter.add(delta, &[]);
}
