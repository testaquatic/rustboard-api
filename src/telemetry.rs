use opentelemetry::{KeyValue, trace::TracerProvider};
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub struct OtelGuard {
    pub tracer_provider: SdkTracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        self.tracer_provider
            .shutdown()
            .unwrap_or_else(|err| eprintln!("OTel TracerProvider shutdown 실패: {err}"))
    }
}

pub fn init_telemetry() -> Result<OtelGuard, anyhow::Error> {
    // 개발/프로덕션 확인
    let is_dev = cfg!(debug_assertions);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if is_dev {
            "rustboard_api=debug,tower_http=debug,sqlx=info".into()
        } else {
            "rustboard_api=info,tower_http=info,sqlx=warn".into()
        }
    });

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
        .with_resource(resource)
        .build();

    // Otel Layer
    let tracer = tracer_provider.tracer("rustboard_api");
    let otel_layer = OpenTelemetryLayer::new(tracer);

    // 합성
    tracing_subscriber::registry()
        .with(env_filter)
        .with(dev_fmt)
        .with(prod_fmt)
        .with(otel_layer)
        .init();

    Ok(OtelGuard { tracer_provider })
}
