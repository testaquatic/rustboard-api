use opentelemetry::{KeyValue, trace::TracerProvider};
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig};
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
    pub tracer_provider: SdkTracerProvider,
    pub meter_provider: SdkMeterProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Otel TracerProvider shutdown 실패: {}", err);
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Otel MeterProvider shutdown 실패: {}", err);
        }
    }
}

pub fn init_telemetry(configuration: &Settings) -> Result<OtelGuard, anyhow::Error> {
    // Envfilter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "rustboard_api=debug,tower_http=debug,sqlx=info"
        } else {
            "rustboard_api=info,tower_http=info,sqlx=warn"
        }
        .into()
    });

    // fmt layer
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

    // Otel TracerProvider
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&configuration.otel_exporter_otlp_endpoint)
        .build()?;
    let resource = Resource::builder()
        .with_attributes([KeyValue::new(SERVICE_NAME, "rustboard_api")])
        .build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.clone())
        .build();

    // Otel layer
    let tracer = tracer_provider.tracer("rustboard_api");
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // 메트릭 exporter
    let metric_exporter = MetricExporter::builder().with_tonic().build()?;

    // 주기적 reader
    let metric_reader = PeriodicReader::builder(metric_exporter).build();

    // MetricProvider
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(metric_reader)
        .with_resource(resource)
        .build();

    // 전역 MeterProvider 등록
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
