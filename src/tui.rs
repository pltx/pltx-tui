use std::{
    io::{self, stdout, Stdout},
    time::Instant,
};

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};
use tracing::info;

use crate::keybinds::EventHandler;

type TuiType = Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    pub terminal: TuiType,
    pub events: EventHandler,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let start = Instant::now();
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen)?;
        terminal.clear()?;

        let tui = Self {
            terminal,
            events: EventHandler::init(),
        };

        info!("initialized terminal backend in {:?}", start.elapsed());
        Ok(tui)
    }

    pub fn restore() -> io::Result<()> {
        let start = Instant::now();
        terminal::disable_raw_mode()?;
        crossterm::execute!(stdout(), LeaveAlternateScreen)?;
        info!("restored the terminal in {:?}", start.elapsed());
        Ok(())
    }
}
