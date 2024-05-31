use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, KeyEventHandler};
use ratatui::{layout::Rect, widgets::Widget, Frame};

pub struct Scrollable {
    pub focused: usize,
    focused_prev: usize,
    from_top: usize,
    row_count: RefCell<usize>,
    row_height: u16,
    pub col_lengths: Option<Vec<u16>>,
    area_height: RefCell<u16>,
}

impl Default for Scrollable {
    fn default() -> Self {
        Self {
            from_top: 0,
            focused: 0,
            focused_prev: 0,
            row_count: RefCell::new(0),
            row_height: 1,
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

    pub fn row_height(mut self, height: u16) -> Self {
        self.row_height = height;
        self
    }
}

impl KeyEventHandler for Scrollable {
    fn key_event_handler(&mut self, _: &mut App, key_event: KeyEvent) {
        let header_height = if self.col_lengths.is_some() { 1 } else { 0 };
        let area_height = *self.area_height.borrow() as usize / self.row_height as usize;

        match key_event.code {
            KeyCode::Char('j') => {
                if self.focused != self.row_count.borrow().saturating_sub(1) {
                    let is_focus_row_end = self.focused
                        == self.from_top + area_height.saturating_sub(1 + header_height);
                    if is_focus_row_end {
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
                    .saturating_sub(area_height - header_height);
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
        let row_layouts = self.row_rects(area);

        for (i, row) in table.into_iter().enumerate().filter(|(ri, _)| {
            (self.from_top..self.from_top + (area.height as usize / self.row_height as usize))
                .contains(ri)
        }) {
            frame.render_widget(row, row_layouts[i - self.from_top]);
        }
    }

    pub fn render_with_cols<T>(
        &self,
        frame: &mut Frame,
        area: Rect,
        header: Vec<T>,
        table: Vec<Vec<T>>,
    ) where
        T: Widget,
    {
        *self.area_height.borrow_mut() = area.height;
        *self.row_count.borrow_mut() = table.len();
        let row_layouts = self.row_rects(area);

        if let Some(col_lengths) = &self.col_lengths {
            let row_area = row_layouts[0];
            for (i, widget) in header.into_iter().enumerate() {
                let col_area = Rect::new(
                    row_area.x + col_lengths[0..i].iter().fold(0, |sum, l| sum + *l),
                    row_area.y,
                    col_lengths[i],
                    row_area.height,
                );
                frame.render_widget(widget, col_area);
            }

            for (ri, rows) in table.into_iter().enumerate().filter(|(ri, _)| {
                let header_height = 1;
                (self.from_top
                    ..self.from_top + (area.height as usize / self.row_height as usize)
                        - header_height)
                    .contains(ri)
            }) {
                let row_layout = row_layouts[ri + 1 - self.from_top];

                for (ci, widget) in rows.into_iter().enumerate() {
                    let col_layout = Rect::new(
                        row_layout.x + col_lengths[0..ci].iter().fold(0, |sum, l| sum + *l),
                        row_layout.y,
                        col_lengths[ci],
                        row_layout.height,
                    );
                    frame.render_widget(widget, col_layout);
                }
            }
        } else {
            panic!("tried to render scrollable with cols, but col_lengths not defined")
        }
    }

    fn row_rects(&self, area: Rect) -> Vec<Rect> {
        (0..area.height / self.row_height)
            .map(|i| {
                Rect::new(
                    area.x,
                    area.y + (i * self.row_height),
                    area.width,
                    self.row_height,
                )
            })
            .collect::<Vec<Rect>>()
    }

    pub fn reset(&mut self) {
        self.focused = 0;
        self.focused_prev = 0;
        self.from_top = 0;
    }
}
