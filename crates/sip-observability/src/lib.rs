use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the global tracing subscriber.
///
/// Uses `SIP_LOG_FORMAT=json` to select JSON output; otherwise uses pretty formatting.
/// The log level is controlled by the `RUST_LOG` environment variable via `EnvFilter`.
pub fn init_tracing() -> anyhow::Result<()> {
    let format = std::env::var("SIP_LOG_FORMAT").unwrap_or_default();
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    if format.eq_ignore_ascii_case("json") {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }

    Ok(())
}

/// Generate a new UUID v4 string to use as a request identifier.
pub fn make_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Record the given `request_id` in the current tracing span.
pub fn record_request_id(request_id: &str) {
    tracing::Span::current().record("request_id", request_id);
}
