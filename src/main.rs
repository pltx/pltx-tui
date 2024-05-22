use clap::{Parser, Subcommand};
use pltx::{errors, run_tui};
use pltx_app::App;
use pltx_config::get_config;
use pltx_tracing::initialize_logging;

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
    about = "TUI for pltx. Manage your personal life privately and securely.",
    help_template(HELP_TEMPLATE)
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Delete all existing data
    Reset,
}

fn main() -> color_eyre::eyre::Result<()> {
    errors::install_hooks()?;
    let config = get_config();
    initialize_logging(&config.log_level)?;

    let cli = Cli::parse();
    let mut app = App::new(config);

    match &cli.command {
        Some(Commands::Reset) => {
            app.db.reset();
            println!("Ok")
        }
        None => {
            run_tui(&mut app)?;
        }
    }

    Ok(())
}
