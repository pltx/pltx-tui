use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::{
    components,
    state::State,
    utils::{Init, KeyEventHandler, RenderPopup},
    App,
};

pub struct CreateProject {
    pub width: u16,
    pub height: u16,
}

impl Init for CreateProject {
    fn init(_: &mut crate::App) -> CreateProject {
        CreateProject {
            width: 70,
            height: 20,
        }
    }
}

impl KeyEventHandler for CreateProject {
    fn key_event_handler(&mut self, _: &mut crate::App, _: KeyEvent, _: &State) {}
}

impl RenderPopup for CreateProject {
    fn render(&mut self, frame: &mut Frame, app: &mut App) {
        components::Popup::new(app, frame.size())
            .set_title("Create Project")
            .render(frame);
    }
}
