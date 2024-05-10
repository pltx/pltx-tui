use std::io::{self, stdout, Stdout};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

use crate::trace_info;

/// A type alias for the terminal type used in this application.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal.
pub fn init() -> io::Result<Tui> {
    trace_info!("entering alternate screen");
    execute!(stdout(), EnterAlternateScreen)?;
    trace_info!("enabling raw mode");
    enable_raw_mode()?;
    trace_info!("create new crossterm instance");
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

/// Restore the terminal to its original state.
pub fn restore() -> io::Result<()> {
    trace_info!("leaving alternate screen");
    execute!(stdout(), LeaveAlternateScreen)?;
    trace_info!("disabling raw mode");
    disable_raw_mode()?;
    Ok(())
}
