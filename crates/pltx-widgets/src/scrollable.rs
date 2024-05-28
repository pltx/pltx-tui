use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, KeyEventHandler};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
    Frame,
};

pub struct Scrollable {
    pub focused: usize,
    focused_prev: usize,
    from_top: usize,
    row_count: RefCell<usize>,
    col_lengths: Vec<u16>,
    area_height: RefCell<u16>,
}

impl Scrollable {
    pub fn new<const N: usize>(col_lengths: [u16; N]) -> Self {
        Self {
            from_top: 0,
            focused: 0,
            focused_prev: 0,
            row_count: RefCell::new(0),
            col_lengths: col_lengths.into(),
            area_height: RefCell::new(0),
        }
    }
}

impl KeyEventHandler for Scrollable {
    fn key_event_handler(&mut self, app: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => {
                if self.focused != self.row_count.borrow().saturating_sub(1) {
                    tracing::debug!(
                        "focused = {}, from_top = {}, height = {}",
                        self.focused,
                        self.from_top,
                        *self.area_height.borrow()
                    );
                    if self.focused + 1 == self.from_top + *self.area_height.borrow() as usize - 1 {
                        self.from_top += 1;
                    }
                    self.focused_prev = self.focused;
                    self.focused += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.focused != 0 {
                    if self.focused == self.from_top {
                        self.from_top -= 1;
                    }
                    self.focused_prev = self.focused;
                    self.focused -= 1;
                }
            }
            KeyCode::Char('g') => {
                self.from_top = 0;
                self.focused_prev = 0;
                self.focused = 0;
            }
            KeyCode::Char('G') => {
                self.from_top = self
                    .row_count
                    .borrow()
                    .saturating_sub(*self.area_height.borrow() as usize - 1);
                self.focused_prev = 0;
                self.focused = self.row_count.borrow().saturating_sub(1);
            }
            KeyCode::Char('d') => {
                if self.focused != 0 {
                    app.delete_mode()
                }
            }
            KeyCode::Char('y') => app.normal_mode(),
            KeyCode::Char('n') => app.normal_mode(),
            _ => {}
        }
    }
}

// TODO: Allow a Row widget or something similar to be passed as rows, so the user doesn't have to
// specify a row style for each cell.
impl Scrollable {
    pub fn render<const N: usize>(
        &self,
        frame: &mut Frame,
        area: Rect,
        header: [impl Widget; N],
        table: Vec<[impl Widget; N]>,
    ) {
        *self.area_height.borrow_mut() = area.height;
        *self.row_count.borrow_mut() = table.len();

        let row_layouts = Layout::default()
            .constraints(
                (0..area.height)
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<Constraint>>(),
            )
            .split(area);

        let header_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                (0..header.len())
                    .map(|i| Constraint::Length(self.col_lengths[i]))
                    .collect::<Vec<Constraint>>(),
            )
            .split(row_layouts[0]);

        for (i, widget) in header.into_iter().enumerate() {
            frame.render_widget(widget, header_layout[i]);
        }

        for (ri, cols) in table.into_iter().enumerate().filter(|(ri, _)| {
            (self.from_top..self.from_top + area.height as usize - 1).contains(ri)
        }) {
            let col_layouts = Layout::default()
                .direction(Direction::Horizontal)
                .constraints((0..cols.len()).map(|i| Constraint::Length(self.col_lengths[i])))
                .split(row_layouts[ri + 1 - self.from_top]);

            for (ci, widget) in cols.into_iter().enumerate() {
                frame.render_widget(widget, col_layouts[ci]);
            }
        }
    }
}
