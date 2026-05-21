use opentelemetry::{KeyValue, trace::TracerProvider};
use opentelemetry_otlp::{MetricExporter, SpanExporter};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
    trace::SdkTracerProvider,
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub struct OtelGuard {
    pub tracer_provider: SdkTracerProvider,
    pub meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        self.tracer_provider
            .shutdown()
            .unwrap_or_else(|err| eprintln!("OTel TracerProvider shutdown 실패: {err}"));
        self.meter_provider
            .shutdown()
            .unwrap_or_else(|err| eprintln!("OTel MeterProvider shutdown 실패: {err}"));
    }
}

pub fn init_telemetry(env_filter: EnvFilter) -> Result<OtelGuard, anyhow::Error> {
    // 개발/프로덕션 확인
    let is_dev = cfg!(debug_assertions);

    let dev_fmt = if is_dev {
        Some(tracing_subscriber::fmt::layer().pretty().with_target(true))
    } else {
        None
    };

    let prod_fmt = if !is_dev {
        Some(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true),
        )
    } else {
        None
    };

    // Otel TraceProvider
    let exporter = SpanExporter::builder().with_tonic().build()?;
    let resource = Resource::builder()
        .with_attributes([KeyValue::new(SERVICE_NAME, "rustboard_api")])
        .build();
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.clone())
        .build();

    // Otel Layer
    let tracer = tracer_provider.tracer("rustboard_api");
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // 메트릭 exporter
    let metric_exporter = MetricExporter::builder().with_tonic().build()?;

    // 주기적 reader
    let metric_reader = PeriodicReader::builder(metric_exporter).build();

    // MetricProvider
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(metric_reader)
        .with_resource(resource.clone())
        .build();

    // 전역 MetricProvider 설정
    opentelemetry::global::set_meter_provider(meter_provider.clone());

    // 합성
    tracing_subscriber::registry()
        .with(env_filter)
        .with(dev_fmt)
        .with(prod_fmt)
        .with(otel_layer)
        .init();

    Ok(OtelGuard {
        tracer_provider,
        meter_provider,
    })
}
