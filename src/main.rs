use pltx::{errors, run, tui};
use pltx_app::App;
use pltx_config::get_config;
use pltx_tracing::initialize_logging;

fn main() -> color_eyre::eyre::Result<()> {
    errors::install_hooks()?;
    let config = get_config();
    initialize_logging(&config.log_level)?;

    let mut terminal = tui::init()?;

    let mut app = App::new(config);
    run(&mut terminal, &mut app)?;

    tui::restore()?;

    Ok(())
}
