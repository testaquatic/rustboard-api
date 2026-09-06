use opentelemetry::{KeyValue, trace::TracerProvider};
use opentelemetry_otlp::{ExporterBuildError, MetricExporter, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::{PeriodicReader, SdkMeterProvider},
    trace::SdkTracerProvider,
};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::configuration::Settings;

pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Otel TracerProvider shutdown error: {}", err)
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Otel MeterProvider shutdown error: {}", err)
        }
    }
}

/// 로깅을 시작한다.
pub fn init_telemetry(settings: &Settings) -> Result<OtelGuard, ExporterBuildError> {
    // EnvFilter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "rustboard_api=debug,tower_http=debug,sqlx=info"
        } else {
            "rustboard_api=info,tower_http=info,sqlx=warn"
        }
        .into()
    });

    // fmt filter
    let dev_fmt = if cfg!(debug_assertions) {
        Some(tracing_subscriber::fmt::layer().pretty().with_target(true))
    } else {
        None
    };
    let prod_fmt = if cfg!(not(debug_assertions)) {
        Some(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true),
        )
    } else {
        None
    };

    // Otel layer
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&settings.otel_exporter_otlp_endpoint)
        .build()?;
    let resource = Resource::builder()
        .with_attributes([KeyValue::new(SERVICE_NAME, "rustboard_api")])
        .build();
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.clone())
        .build();
    let tracer = tracer_provider.tracer("rustboard_api");
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // 전역 MeterProvider 설정
    let metric_exporter = MetricExporter::builder().with_tonic().build()?;
    let metric_reader = PeriodicReader::builder(metric_exporter).build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(metric_reader)
        .with_resource(resource)
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());

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
