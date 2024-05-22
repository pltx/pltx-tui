use std::path::PathBuf;

use color_eyre::eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn initialize_logging(log_level: &str) -> Result<()> {
    let home_dir = match home::home_dir() {
        Some(path) => path,
        None => {
            panic!("failed to find home directory");
        }
    };
    let cache_dir_str = format!("{}/.cache/pltx", home_dir.to_str().unwrap());
    let cache_dir = PathBuf::from(cache_dir_str);
    std::fs::create_dir_all(cache_dir.clone())?;
    let log_path = cache_dir.join("debug.log");
    let log_file = std::fs::File::create(log_path)?;
    std::env::set_var("RUST_LOG", log_level.to_uppercase());
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

#[macro_export]
macro_rules! trace_debug {
    ($ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: module_path!(), tracing::Level::DEBUG, ?value);
                value
            }
        }
    }};
}

#[macro_export]
macro_rules! trace_info {
    ($ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: module_path!(), tracing::Level::INFO, ?value);
                value
            }
        }
    }};
}

#[macro_export]
macro_rules! trace_warn {
    ($ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: module_path!(), tracing::Level::WARN, ?value);
                value
            }
        }
    }};
}

#[macro_export]
macro_rules! trace_error {
    ($ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: module_path!(), tracing::Level::ERROR, ?value);
                value
            }
        }
    }};
}

#[macro_export]
macro_rules! trace_panic {
    ($value:expr) => {{
        tracing::event!(target: module_path!(), tracing::Level::ERROR, $value);
        panic!($value);
    }};
}
