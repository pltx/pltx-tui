use color_eyre::Result;

use pltx_tui::{errors, tui, App};

fn main() -> Result<()> {
    errors::install_hooks()?;

    let mut terminal = tui::init()?;

    let mut app = App::default();
    app.run(&mut terminal)?;

    tui::restore()?;

    Ok(())
}
