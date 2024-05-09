use pltx::{config::get_config, errors, tracing::initialize_logging, tui, App};

fn main() -> color_eyre::eyre::Result<()> {
    errors::install_hooks()?;
    let config = get_config();
    initialize_logging(&config.log_level)?;

    let mut terminal = tui::init()?;

    let mut app = App::new(config);
    app.run(&mut terminal)?;

    tui::restore()?;

    Ok(())
}
