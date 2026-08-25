use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_subscriber() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "rustboard_api=debug,tower_http=debug,sqlx=info"
        } else {
            "rustboard_api=info,tower_http=info,sqlx=warn"
        }
        .into()
    });

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

    tracing_subscriber::registry()
        .with(env_filter)
        .with(dev_fmt)
        .with(prod_fmt)
        .init();
}
