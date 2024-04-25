use pltx::{errors, tracing::initialize_logging, tui, App};

fn main() -> color_eyre::eyre::Result<()> {
    errors::install_hooks()?;
    initialize_logging()?;

    let mut terminal = tui::init()?;

    let mut app = App::new();
    app.run(&mut terminal)?;

    tui::restore()?;

    Ok(())
}
