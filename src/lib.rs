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
use ui::Interface;

/// Runs the application's main loop until the user quits.
pub fn run(terminal: &mut tui::Tui, app: &mut App) -> color_eyre::eyre::Result<()> {
    app.db.ensure_tables().unwrap_or_else(|e| panic!("{e}"));
    app.db.insert_session().unwrap_or_else(|e| panic!("{e}"));
    let mut interface = Interface::init(app);
    interface.init_data(app).unwrap_or_else(|e| panic!("{e}"));
    let mut command_handler = CommandHandler::new();
    let mut event_handler = EventHandler::init();

    while !app.state.exit {
        terminal.draw(|frame| {
            interface.render(frame, app, &mut command_handler);
        })?;

        event_handler
            .handle_events(app, &mut interface, &mut command_handler)
            .wrap_err("handle events failed")?;
    }
    Ok(())
}
