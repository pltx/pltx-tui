//! pltx
use std::time::Instant;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use pltx::{errors, run_tui};
use pltx_app::App;
use pltx_config::{init_config, ProfileConfig};
use pltx_utils::dirs;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

const VERSION: &str = env!("CARGO_PKG_VERSION");
static HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author}
{about}

{usage-heading}
  {usage}

{all-args}{after-help}";

#[derive(Parser, Debug)]
#[command(
    name = "pltx",
    version = VERSION,
    author = "Slekup <opensource@slekup.com>",
    about = "Manage your personal life privately and securely in the terminal.",
    help_template(HELP_TEMPLATE)
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Use a profile
    #[arg(short, long)]
    profile: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Delete all existing data
    Reset,
}

fn main() -> Result<()> {
    let application_start = Instant::now();

    errors::install_hooks()?;
    let cli = Cli::parse();
    let (config, profile) = init_config(cli.profile.clone())?;
    init_tracing(&config.log_level, &profile)?;

    let mut app = App::new(config, profile);

    match &cli.command {
        Some(Commands::Reset) => {
            app.db.reset()?;
            println!("Ok {:?}", application_start.elapsed());
        }
        None => {
            run_tui(&mut app, application_start)?;
        }
    }

    Ok(())
}

/// Initialize the tracing crate.
pub fn init_tracing(log_level: &str, profile: &ProfileConfig) -> Result<()> {
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
