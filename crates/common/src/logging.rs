use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    
    // We default to pretty logs for CLI usage.
    // In the future, we can switch to JSON based on an env var or flag.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
