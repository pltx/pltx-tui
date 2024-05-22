use color_eyre::eyre::WrapErr;
use pltx_app::App;

mod command_handler;
pub mod errors;
mod keybinds;
mod popups;
pub mod tui;
mod ui;

use command_handler::CommandHandler;
use keybinds::EventHandler;
use pltx_utils::Popup;
use ui::Interface;

/// Runs the tui application's main loop until the user quits.
pub fn run_tui(app: &mut App) -> color_eyre::eyre::Result<()> {
    let mut terminal = tui::init()?;
    app.db.ensure_tables().unwrap_or_else(|e| panic!("{e}"));
    app.db.insert_session().unwrap_or_else(|e| panic!("{e}"));
    let mut interface = Interface::init(app);
    let mut command_handler = CommandHandler::init(app);
    let mut event_handler = EventHandler::init();

    while !app.exit {
        terminal.draw(|frame| {
            interface.render(frame, app, &mut command_handler);
        })?;

        event_handler
            .handle_events(app, &mut interface, &mut command_handler)
            .wrap_err("handle events failed")?;
    }

    tui::restore()?;
    Ok(())
}
