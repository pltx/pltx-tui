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

#[derive(PartialEq)]
pub enum CardBorderType {
    Plain,
    Bold,
    Rounded,
}

/// Card widget
pub struct Card {
    title: String,
    focused_title: bool,
    area: Rect,
    margin: WidgetMargin,
    child_margin: WidgetMargin,
    /// TODO: Use ratatui's BorderType instead after updating symbols.
    border_type: CardBorderType,
}

impl Card {
    pub fn new(title: &str, area: Rect) -> Self {
        Self {
            title: title.to_string(),
            focused_title: false,
            area,
            margin: WidgetMargin::default(),
            child_margin: WidgetMargin::default(),
            border_type: CardBorderType::Rounded,
        }
    }

    pub fn focused_title(mut self, focused: bool) -> Self {
        self.focused_title = focused;
        self
    }

    pub fn margin(mut self, margin: WidgetMargin) -> Self {
        self.margin = margin;
        self
    }

    pub fn child_margin(mut self, margin: WidgetMargin) -> Self {
        self.child_margin = margin;
        self
    }

    pub fn border_type(mut self, border_type: CardBorderType) -> Self {
        self.border_type = border_type;
        self
    }

    pub fn child_layout(&self) -> Rect {
        let [_, block_bottom_layout] = Layout::default()
            .constraints([
                Constraint::Length(3 + self.child_margin.top),
                Constraint::Fill(1),
            ])
            .areas(self.margin.apply(self.area));

        let [center_layout, _] = Layout::default()
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1 + self.child_margin.bottom),
            ])
            .areas(block_bottom_layout);

        let [_, child_layout, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1 + self.child_margin.left),
                Constraint::Fill(1),
                Constraint::Length(1 + self.child_margin.right),
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
            .constraints([
                Constraint::Length(3 + self.child_margin.top),
                Constraint::Fill(1),
            ])
            .areas(self.margin.apply(area));

        let mut title_paragraph = Paragraph::new(vec![
            Line::from(vec![
                Span::from(if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::TOP_LEFT
                } else if self.border_type == CardBorderType::Rounded {
                    symbols::border::TOP_LEFT_ROUNDED
                } else {
                    symbols::border::TOP_LEFT
                }),
                Span::from(
                    (if self.border_type == CardBorderType::Bold {
                        symbols::bold::border::HORIZONTAL
                    } else {
                        symbols::border::HORIZONTAL
                    })
                    .repeat((title_layout.width as usize).saturating_sub(2)),
                ),
                Span::from(if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::TOP_RIGHT
                } else if self.border_type == CardBorderType::Rounded {
                    symbols::border::TOP_RIGHT_ROUNDED
                } else {
                    symbols::border::TOP_RIGHT
                }),
            ]),
            Line::from(vec![
                Span::from(if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::VERTICAL
                } else {
                    symbols::border::VERTICAL
                }),
                {
                    let mut title_span = Span::from(format!(
                        " {}{} ",
                        self.title,
                        " ".repeat(
                            (title_layout.width as usize)
                                .saturating_sub(self.title.chars().count() + 4)
                        )
                    ));
                    if focused && self.focused_title {
                        title_span = title_span.bold().bg(colors.input_focus_bg);
                    }
                    title_span
                }
                .fg(colors.fg),
                Span::from(if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::VERTICAL
                } else {
                    symbols::border::VERTICAL
                }),
            ]),
            Line::from(vec![
                Span::from(if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::LEFT_T
                } else {
                    symbols::border::LEFT_T
                }),
                Span::from(
                    if self.border_type == CardBorderType::Bold {
                        symbols::bold::border::HORIZONTAL
                    } else {
                        symbols::border::HORIZONTAL
                    }
                    .repeat((title_layout.width as usize).saturating_sub(2)),
                ),
                Span::from(if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::RIGHT_T
                } else {
                    symbols::border::RIGHT_T
                }),
            ]),
        ])
        .fg(border_color);

        if focused && self.focused_title {
            title_paragraph = title_paragraph.fg(colors.fg);
        }

        frame.render_widget(title_paragraph, title_layout);

        let [center_layout, bottom_line_layout] = Layout::default()
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1 + self.child_margin.bottom),
            ])
            .areas(block_bottom_layout);

        let [left_border_layout, _, right_border_layout] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1 + self.child_margin.left),
                Constraint::Fill(1),
                Constraint::Length(1 + self.child_margin.right),
            ])
            .areas(center_layout);

        let left_border = Paragraph::new(
            (0..center_layout.height as usize)
                .map(|_| {
                    Line::from(format!(
                        "{}{}",
                        (if self.border_type == CardBorderType::Bold {
                            symbols::bold::border::VERTICAL
                        } else {
                            symbols::border::VERTICAL
                        }),
                        " ".repeat(self.child_margin.left as usize)
                    ))
                })
                .collect::<Vec<Line>>(),
        )
        .fg(border_color);

        let right_border = Paragraph::new(
            (0..center_layout.height as usize)
                .map(|_| {
                    Line::from(format!(
                        "{}{}",
                        " ".repeat(self.child_margin.right as usize),
                        (if self.border_type == CardBorderType::Bold {
                            symbols::bold::border::VERTICAL
                        } else {
                            symbols::border::VERTICAL
                        })
                    ))
                })
                .collect::<Vec<Line>>(),
        )
        .fg(border_color);

        frame.render_widget(left_border, left_border_layout);
        frame.render_widget(right_border, right_border_layout);

        let bottom_line = Paragraph::new(Line::from(vec![
            Span::from(if self.border_type == CardBorderType::Bold {
                symbols::bold::border::BOTTOM_LEFT
            } else if self.border_type == CardBorderType::Rounded {
                symbols::border::BOTTOM_LEFT_ROUNDED
            } else {
                symbols::border::BOTTOM_LEFT
            }),
            Span::from(
                (if self.border_type == CardBorderType::Bold {
                    symbols::bold::border::HORIZONTAL
                } else {
                    symbols::border::HORIZONTAL
                })
                .repeat((bottom_line_layout.width as usize).saturating_sub(2)),
            ),
            Span::from(if self.border_type == CardBorderType::Bold {
                symbols::bold::border::BOTTOM_RIGHT
            } else if self.border_type == CardBorderType::Rounded {
                symbols::border::BOTTOM_RIGHT_ROUNDED
            } else {
                symbols::border::BOTTOM_RIGHT
            }),
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
