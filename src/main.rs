use tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default to INFO level, can be overridden with RUST_LOG env var
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("qmkview=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    qmkview_gui::run()
}
