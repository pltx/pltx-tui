use std::rc::Rc;

use pltx_app::{App, DefaultWidget};
use pltx_utils::{symbols, WidgetMargin};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Card widget
pub struct Card {
    title: String,
    area: Rect,
    margin: WidgetMargin,
}

impl Card {
    pub fn new(title: &str, area: Rect) -> Self {
        Self {
            title: title.to_string(),
            area,
            margin: WidgetMargin::default(),
        }
    }

    pub fn margin(mut self, margin: WidgetMargin) -> Self {
        self.margin = margin;
        self
    }
    

    pub fn child_layout(&self) -> Rect {
        let [_, block_bottom_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(self.margin.apply(self.area));

        let [center_layout, _] = Layout::default()
            .constraints([
                Constraint::Length(block_bottom_layout.height.saturating_sub(1)),
                Constraint::Length(1),
            ])
            .areas(block_bottom_layout);

        let [_, child_layout, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(center_layout.width.saturating_sub(4)),
                Constraint::Length(2),
            ])
            .areas(center_layout);

        child_layout
    }
}

impl DefaultWidget for Card {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool) {
        let colors = &app.config.colors;
        let border_color = if focused {
            colors.border_active
        } else {
            colors.border
        };

        let [title_layout, block_bottom_layout] = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .areas(self.margin.apply(area));

        let title_paragraph = Paragraph::new(vec![
            Line::from(vec![
                Span::from(symbols::border::TOP_LEFT),
                Span::from(
                    symbols::border::HORIZONTAL
                        .repeat((title_layout.width as usize).saturating_sub(2)),
                ),
                Span::from(symbols::border::TOP_RIGHT),
            ]),
            Line::from(vec![
                Span::from(symbols::border::VERTICAL),
                Span::from(format!(
                    " {}{} ",
                    self.title,
                    " ".repeat(
                        (title_layout.width as usize)
                            .saturating_sub(self.title.chars().count() + 4)
                    )
                ))
                .fg(colors.fg),
                Span::from(symbols::border::VERTICAL),
            ]),
            Line::from(vec![
                Span::from(symbols::border::LEFT_T),
                Span::from(
                    symbols::border::HORIZONTAL
                        .repeat((title_layout.width as usize).saturating_sub(2)),
                ),
                Span::from(symbols::border::RIGHT_T),
            ]),
        ])
        .fg(border_color);

        frame.render_widget(title_paragraph, title_layout);

        let [center_layout, bottom_line_layout] = Layout::default()
            .constraints([
                Constraint::Length(block_bottom_layout.height.saturating_sub(1)),
                Constraint::Length(1),
            ])
            .areas(block_bottom_layout);

        let [left_border_layout, _, right_border_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(center_layout.width.saturating_sub(4)),
                Constraint::Length(2),
            ])
            .areas(center_layout);

        let left_border = Paragraph::new(
            (0..center_layout.height as usize)
                .map(|_| Line::from(format!("{} ", symbols::border::VERTICAL)))
                .collect::<Vec<Line>>(),
        )
        .fg(border_color);

        let right_border = Paragraph::new(
            (0..center_layout.height as usize)
                .map(|_| Line::from(format!(" {}", symbols::border::VERTICAL)))
                .collect::<Vec<Line>>(),
        )
        .fg(border_color);

        frame.render_widget(left_border, left_border_layout);
        frame.render_widget(right_border, right_border_layout);

        let bottom_line = Paragraph::new(Line::from(vec![
            Span::from(symbols::border::BOTTOM_LEFT),
            Span::from(
                symbols::border::HORIZONTAL
                    .repeat((bottom_line_layout.width as usize).saturating_sub(2)),
            ),
            Span::from(symbols::border::BOTTOM_RIGHT),
        ]))
        .fg(border_color);

        frame.render_widget(bottom_line, bottom_line_layout);
    }
}

pub struct CardCell {
    title: String,
    focused: bool,
    constraint: Option<Constraint>,
}

impl CardCell {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            focused: false,
            constraint: None,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = Some(constraint);
        self
    }
}

pub struct CardRow {
    margin: WidgetMargin,
    cards: Vec<CardCell>,
    card_margin: WidgetMargin,
    height: Option<u16>,
}

impl CardRow {
    pub fn new(cards: Vec<CardCell>) -> Self {
        Self {
            margin: WidgetMargin::default(),
            cards,
            card_margin: WidgetMargin::default(),
            height: None,
        }
    }

    pub fn margin(mut self, margin: WidgetMargin) -> Self {
        self.margin = margin;
        self
    }

    pub fn card_margin(mut self, card_margin: WidgetMargin) -> Self {
        self.card_margin = card_margin;
        self
    }

    /// The height of the card child content will be `height - vertical margin -
    /// 4`.
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    pub fn card_layouts(&self, area: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                self.cards
                    .iter()
                    .map(|c| c.constraint.unwrap_or(Constraint::Fill(1)))
                    .collect::<Vec<Constraint>>(),
            )
            .split(self.margin.apply(area))
    }

    pub fn layouts(&self, area: Rect) -> Vec<Rect> {
        self.cards
            .iter()
            .enumerate()
            .map(|(i, c)| {
                Card::new(&c.title, self.card_layouts(area)[i])
                    .margin(self.card_margin)
                    .child_layout()
            })
            .collect::<Vec<Rect>>()
    }
}

impl DefaultWidget for CardRow {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool) {
        let card_layouts = self.card_layouts(area);
        for (i, card_cell) in self.cards.iter().enumerate() {
            let card = Card::new(&card_cell.title, card_layouts[i]).margin(self.card_margin);
            card.render(frame, app, card_layouts[i], focused && card_cell.focused);
        }
    }
}

pub struct CardLayout<const N: usize> {
    rows: [CardRow; N],
    max_width: Option<u16>,
    margin: WidgetMargin,
}

impl<const N: usize> CardLayout<N> {
    pub fn new(rows: [CardRow; N]) -> Self {
        Self {
            rows,
            max_width: None,
            margin: WidgetMargin::default(),
        }
    }

    pub fn max_width(mut self, max_width: u16) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn margin(mut self, margin: WidgetMargin) -> Self {
        self.margin = margin;
        self
    }

    pub fn row_margin(mut self, row_margin: WidgetMargin) -> Self {
        for row in self.rows.iter_mut() {
            row.margin = row_margin;
        }
        self
    }

    pub fn card_margin(mut self, card_margin: WidgetMargin) -> Self {
        for row in self.rows.iter_mut() {
            row.card_margin = card_margin;
        }
        self
    }

    fn row_layouts(&self, area: Rect) -> Rc<[Rect]> {
        let [_, max_width_layout, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                if let Some(max_width) = self.max_width {
                    Constraint::Max(max_width)
                } else {
                    Constraint::Fill(1)
                },
                Constraint::Fill(1),
            ])
            .areas(self.margin.apply(area));

        Layout::default()
            .constraints(
                self.rows
                    .iter()
                    .map(|r| {
                        if let Some(height) = r.height {
                            Constraint::Length(height)
                        } else {
                            Constraint::Fill(1)
                        }
                    })
                    .collect::<Vec<Constraint>>(),
            )
            .split(max_width_layout)
    }

    // TODO: More idiomatic way of access the card layouts from the row vectors than
    // row_layout[i]
    pub fn layouts<const R: usize>(&self, area: Rect) -> [Vec<Rect>; R] {
        let row_layouts = self.row_layouts(area);

        self.rows
            .iter()
            .enumerate()
            .map(|(i, r)| r.layouts(row_layouts[i]))
            .collect::<Vec<Vec<Rect>>>()
            .try_into()
            .expect("invalid number of row layouts")
    }
}

impl<const N: usize> DefaultWidget for CardLayout<N> {
    fn render(&self, frame: &mut Frame, app: &App, area: Rect, focused: bool) {
        let row_layouts = self.row_layouts(area);
        for (i, row) in self.rows.iter().enumerate() {
            row.render(frame, app, row_layouts[i], focused);
        }
    }
}
