use std::io::{self, stdout, Stdout};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

use crate::trace_debug;

/// A type alias for the terminal type used in this application.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal.
pub fn init() -> io::Result<Tui> {
    trace_debug!("entering alternate screen");
    execute!(stdout(), EnterAlternateScreen)?;
    trace_debug!("enabling raw mode");
    enable_raw_mode()?;
    trace_debug!("create new crossterm instance");
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

/// Restore the terminal to its original state.
pub fn restore() -> io::Result<()> {
    trace_debug!("leaving alternate screen");
    execute!(stdout(), LeaveAlternateScreen)?;
    trace_debug!("disabling raw mode");
    disable_raw_mode()?;
    Ok(())
}
