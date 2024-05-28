use clap::{Parser, Subcommand};
use color_eyre::Result;
use pltx::{errors, run_tui, tracing::init_logging};
use pltx_app::App;
use pltx_config::init_config;

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
    errors::install_hooks()?;
    let cli = Cli::parse();
    let (config, profile) = init_config(cli.profile.clone())?;
    init_logging(&config.log_level, &profile)?;

    let mut app = App::new(config, profile);

    match &cli.command {
        Some(Commands::Reset) => {
            app.db.reset()?;
            println!("Ok")
        }
        None => {
            run_tui(&mut app)?;
        }
    }

    Ok(())
}
