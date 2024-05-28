use std::io::{self, stdout, Stdout};

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::keybinds::EventHandler;

type TuiType = Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    pub terminal: TuiType,
    pub events: EventHandler,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen)?;
        terminal.clear()?;

        Ok(Self {
            terminal,
            events: EventHandler::init(),
        })
    }

    pub fn restore() -> io::Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}
