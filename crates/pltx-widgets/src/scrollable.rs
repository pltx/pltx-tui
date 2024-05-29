use std::{cell::RefCell, rc::Rc};

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
    col_lengths: Option<Vec<u16>>,
    area_height: RefCell<u16>,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self {
            from_top: 0,
            focused: 0,
            focused_prev: 0,
            row_count: RefCell::new(0),
            col_lengths: None,
            area_height: RefCell::new(0),
        }
    }
}

impl Scrollable {
    pub fn cols<const N: usize>(mut self, col_lengths: [u16; N]) -> Self {
        self.col_lengths = Some(col_lengths.into());
        self
    }
}

impl KeyEventHandler for Scrollable {
    fn key_event_handler(&mut self, _: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => {
                if self.focused != self.row_count.borrow().saturating_sub(1) {
                    if self.focused + 1
                        == self.from_top + (*self.area_height.borrow() as usize).saturating_sub(1)
                    {
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
            _ => {}
        }
    }
}

// TODO: Allow a Row widget or something similar to be passed as rows, so the
// user doesn't have to specify a row style for each cell.
impl Scrollable {
    pub fn render<T>(&self, frame: &mut Frame, area: Rect, table: Vec<T>)
    where
        T: Widget,
    {
        *self.area_height.borrow_mut() = area.height;
        *self.row_count.borrow_mut() = table.len();
        let row_layouts = self.row_layouts(area);

        for (i, row) in table.into_iter().enumerate().filter(|(ri, _)| {
            (self.from_top..self.from_top + area.height as usize - 1).contains(ri)
        }) {
            frame.render_widget(row, row_layouts[i + 1 - self.from_top]);
        }
    }

    pub fn render_with_cols<T, const N: usize>(
        &self,
        frame: &mut Frame,
        area: Rect,
        header: [T; N],
        table: Vec<[T; N]>,
    ) where
        T: Widget,
    {
        *self.area_height.borrow_mut() = area.height;
        *self.row_count.borrow_mut() = table.len();
        let row_layouts = self.row_layouts(area);

        if let Some(col_lengths) = &self.col_lengths {
            let header_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    (0..header.len())
                        .map(|i| Constraint::Length(col_lengths[i]))
                        .collect::<Vec<Constraint>>(),
                )
                .split(row_layouts[0]);

            for (i, widget) in header.into_iter().enumerate() {
                frame.render_widget(widget, header_layout[i]);
            }

            for (ri, rows) in table.into_iter().enumerate().filter(|(ri, _)| {
                (self.from_top..self.from_top + area.height as usize - 1).contains(ri)
            }) {
                let col_layouts = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints((0..rows.len()).map(|i| Constraint::Length(col_lengths[i])))
                    .split(row_layouts[ri + 1 - self.from_top]);

                for (ci, widget) in rows.into_iter().enumerate() {
                    frame.render_widget(widget, col_layouts[ci]);
                }
            }
        } else {
            panic!("tried to render scrollable with cols, but col_lengths not defined")
        }
    }

    fn row_layouts(&self, area: Rect) -> Rc<[Rect]> {
        Layout::default()
            .constraints(
                (0..area.height)
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<Constraint>>(),
            )
            .split(area)
    }

    pub fn reset(&mut self) {
        self.focused = 0;
        self.focused_prev = 0;
        self.from_top = 0;
    }
}
