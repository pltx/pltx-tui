use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::{
    components,
    state::State,
    utils::{Init, KeyEventHandler, RenderPopup},
    App,
};

pub struct DeleteProject {
    pub width: u16,
    pub height: u16,
}

impl Init for DeleteProject {
    fn init(_: &mut crate::App) -> DeleteProject {
        DeleteProject {
            width: 70,
            height: 20,
        }
    }
}

impl KeyEventHandler for DeleteProject {
    fn key_event_handler(&mut self, _: &mut crate::App, _: KeyEvent, _: &State) {}
}

impl RenderPopup for DeleteProject {
    fn render(&mut self, frame: &mut Frame, app: &App) {
        components::Popup::new(app, frame.size())
            .set_title_top("Create Project")
            .render(frame);
    }
}
