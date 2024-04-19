use crossterm::event::KeyEvent;

use crate::{
    state::State,
    utils::{Init, KeyEventHandler, RenderPopup, RenderScrollPopup},
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
    fn key_event_handler(
        &mut self,
        app: &mut crate::App,
        key_event: KeyEvent,
        event_state: &State,
    ) {
    }
}

impl RenderPopup for CreateProject {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, app: &mut crate::App) {}
}
