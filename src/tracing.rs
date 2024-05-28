use color_eyre::Result;
use pltx_config::ProfileConfig;
use pltx_utils::dirs;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// Initialize the tracing crate.
pub fn init_logging(log_level: &str, profile: &ProfileConfig) -> Result<()> {
    std::env::set_var("RUST_LOG", log_level.to_uppercase());
    let log_path = dirs::cache_dir().join(&profile.log_file);
    let log_file = std::fs::File::create(log_path)?;

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}
