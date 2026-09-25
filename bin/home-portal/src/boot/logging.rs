use tracing_subscriber::EnvFilter;

pub const DEFAULT_LEVEL: &str = "info";

pub fn install() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LEVEL));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
