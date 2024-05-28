//! The main src crate contains code that isn't a dependency of any workspace
//! crates.

use color_eyre::Result;
use keybinds::Event;
use pltx_app::App;

mod command_handler;
pub mod errors;
mod keybinds;
mod popups;
pub mod tracing;
mod tui;
mod ui;

use command_handler::CommandHandler;
use pltx_app::Popup;
use tui::Tui;
use ui::Interface;

/// Initialize and run the terminal user interface
pub fn run_tui(app: &mut App) -> Result<()> {
    let mut tui = Tui::new()?;
    app.db.start_session()?;
    let mut interface = Interface::init(app)?;
    let mut command_handler = CommandHandler::init();

    while !app.exit {
        tui.terminal.draw(|frame| {
            interface.render(frame, app, &mut command_handler);
        })?;

        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                tui.events
                    .key_events(app, &mut interface, &mut command_handler, key_event)?
            }
            // Event::Mouse(_) => {}
            // Event::Resize(_, _) => {}
            Event::FocusGained => {}
            Event::FocusLost => {} // Event::Paste(_) => {}
        }
    }

    Tui::restore()?;
    Ok(())
}
