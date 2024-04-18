use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use crate::{
    state::{Mode, Pane, State},
    utils::{KeyEventHandler, RenderScreen, ScreenKeybinds},
    App,
};

#[derive(PartialEq, Clone)]
enum Tab {
    Planned,
    Projects,
    Important,
}

pub struct ProjectManagement {
    tab: Tab,
    hover_tab: Tab,
}

impl ProjectManagement {
    fn get_tabs<'a>(&self) -> Vec<(Tab, &'a str)> {
        vec![
            (Tab::Planned, "Planned"),
            (Tab::Projects, "Projects"),
            (Tab::Important, "Important"),
        ]
    }
}

impl ScreenKeybinds for ProjectManagement {
    fn screen_keybinds<'a>(&mut self) -> [(&'a str, &'a str); 3] {
        [("n", "New"), ("e", "Edit"), ("d", "Delete")]
    }
}

impl KeyEventHandler for ProjectManagement {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent, _: &State) {
        if app.state.mode == Mode::Navigation && app.state.pane == Pane::Screen {
            let tabs = self.get_tabs();
            let tab_index = tabs.iter().position(|t| t.0 == self.tab).unwrap();
            match key_event.code {
                KeyCode::Char('l') => {
                    if tab_index == tabs.len() - 1 {
                        self.tab = tabs[0].0.clone();
                    } else {
                        self.tab = tabs[tab_index + 1].0.clone();
                    }
                }
                KeyCode::Char('h') => {
                    if tab_index == 0 {
                        self.tab = tabs[tabs.len() - 1].0.clone();
                    } else {
                        self.tab = tabs[tab_index - 1].0.clone();
                    }
                }
                _ => {}
            }
        }
    }
}

impl RenderScreen for ProjectManagement {
    fn init() -> ProjectManagement {
        ProjectManagement {
            tab: Tab::Planned,
            hover_tab: Tab::Planned,
        }
    }

    fn render(&mut self, frame: &mut Frame, app: &App, area: Rect) {
        let colors = &app.config.colors;
        let text = Paragraph::new("Project Management");
        frame.render_widget(text, area);

        let [navigation_layout, content_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(area);

        let navigation_line = vec![Line::from(
            self.get_tabs()
                .iter()
                .map(|t| {
                    let mut style = Style::new();
                    if t.0 == self.tab {
                        style = style.fg(colors.active_fg).bg(colors.active_bg).bold()
                    } else {
                        style = style.fg(colors.secondary)
                    };
                    Span::from(format!(" {} ", t.1)).style(style)
                })
                .collect::<Vec<Span>>(),
        )];

        let navigation = Paragraph::new(navigation_line).block(
            Block::new()
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(colors.primary)),
        );

        frame.render_widget(navigation, navigation_layout);

        let content = Block::new();
        frame.render_widget(content, content_layout)
    }
}
