use std::cell::RefCell;

use ansi_to_tui::IntoText;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use pltx_app::{App, DefaultWidget, Screen};
use pltx_utils::{centered_rect, symbols};
use pltx_widgets::{Card, Scrollable};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};

use crate::generated_docs::DOCUMENTS;

#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub filename: &'static str,
    pub frontmatter: Frontmatter,
    pub content: &'static str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Frontmatter {
    pub title: &'static str,
    pub last_updated: &'static str,
}

#[derive(PartialEq)]
enum Page {
    Selection,
    Document,
}

pub struct Help {
    page: Page,
    selected_page: usize,
    document: Option<&'static Document>,
    scrollable: Scrollable,
    highlighted_content: Option<String>,
    area_height: RefCell<usize>,
    line_count: usize,
    focused: usize,
    focused_prev: usize,
    from_top: usize,
}

impl Screen for Help {
    fn init(_: &App) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            page: Page::Selection,
            selected_page: 0,
            document: None,
            scrollable: Scrollable::default(),
            highlighted_content: None,
            area_height: RefCell::new(0),
            line_count: 0,
            focused: 0,
            focused_prev: 0,
            from_top: 0,
        })
    }

    fn key_event_handler(&mut self, _: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('j') => {
                if self.page == Page::Selection {
                    if self.selected_page != DOCUMENTS.len().saturating_sub(1) {
                        self.selected_page += 1;
                    } else {
                        self.selected_page = 0;
                    }
                } else if self.focused != self.line_count.saturating_sub(1) {
                    if self.focused + 1 == self.from_top + *self.area_height.borrow() {
                        self.from_top += 1;
                    }
                    self.focused_prev = self.focused;
                    self.focused += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.page == Page::Selection {
                    if self.selected_page != 0 {
                        self.selected_page -= 1;
                    } else {
                        self.selected_page = DOCUMENTS.len().saturating_sub(1);
                    }
                } else if self.focused != 0 {
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
                    .line_count
                    .saturating_sub(*self.area_height.borrow() - 1);
                self.focused_prev = 0;
                self.focused = self.line_count.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                let document = &DOCUMENTS[self.selected_page];

                self.scrollable.reset();
                self.document = Some(document);
                self.page = Page::Document;

                // TODO: This is very slow, possibly try to render the content first and load
                // the highlighting in the background?
                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let syntax = ps.find_syntax_by_extension("md").unwrap();
                let mut highlight = HighlightLines::new(syntax, &ts.themes["base16-eighties.dark"]);

                // Skip 1 line which is an empty line.

                self.line_count = document.content.split('\n').count() - 1;

                let highlighted_content = document
                    .content
                    .split('\n')
                    .skip(1)
                    .map(|line| format!("{}\n", line))
                    .map(|line| {
                        syntect::util::as_24_bit_terminal_escaped(
                            &highlight.highlight_line(&line, &ps).unwrap(),
                            false,
                        )
                    })
                    .collect::<String>();

                self.highlighted_content = Some(highlighted_content);
            }
            KeyCode::Char('[') => {
                if self.page == Page::Document {
                    self.page = Page::Selection;
                    self.document = None;
                    self.highlighted_content = None;
                    self.line_count = 0;
                    self.focused = 0;
                    self.focused_prev = 0;
                    self.from_top = 0;
                }
            }
            _ => {}
        }
    }

    fn render(&self, app: &App, frame: &mut Frame, area: Rect) {
        if self.page == Page::Document {
            if let Some(document) = self.document {
                self.render_document(app, frame, area, document);
            }
        } else {
            self.render_selection(app, frame, area);
        }
    }
}

impl Help {
    fn render_document(&self, app: &App, frame: &mut Frame, area: Rect, document: &Document) {
        let colors = &app.config.colors;

        let [frontmatter_layout, content_layout] = Layout::default()
            .horizontal_margin(1)
            .constraints([Constraint::Length(6), Constraint::Fill(1)])
            .areas(area);

        let [side_line_layout, scrollable_content_layout] = Layout::default()
            .horizontal_margin(1)
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(7), Constraint::Fill(1)])
            .areas(content_layout);

        (*self.area_height.borrow_mut()) = content_layout.height as usize;

        let side_line = Paragraph::new(
            (0..content_layout.height as usize)
                .enumerate()
                .map(|(i, _)| {
                    let line_no = i + self.from_top;
                    Line::from(vec![
                        Span::from(" "),
                        Span::from((line_no + 1).to_string()),
                        Span::from(
                            " ".repeat(4usize.saturating_sub((line_no + 1).to_string().len())),
                        ),
                        Span::from(symbols::border::VERTICAL)
                            .fg(colors.tertiary_fg)
                            .bg(colors.bg),
                    ])
                    .fg(if self.focused == line_no {
                        colors.bg
                    } else {
                        colors.tertiary_fg
                    })
                    .bg(if self.focused == line_no {
                        colors.fg
                    } else {
                        colors.bg
                    })
                })
                .collect::<Vec<Line>>(),
        );

        frame.render_widget(side_line, side_line_layout);

        let frontmatter = Paragraph::new(vec![
            Line::from(vec![
                Span::from("Title: ").fg(colors.secondary_fg),
                Span::from(document.frontmatter.title),
            ]),
            Line::from(vec![
                Span::from("Last Updated: ").fg(colors.secondary_fg),
                Span::from(document.frontmatter.last_updated),
            ]),
            Line::from(vec![
                Span::from("Filename: ").fg(colors.secondary_fg),
                Span::from(format!("{}.md", document.filename)),
            ]),
            Line::from(vec![
                Span::from("GitHub: ").fg(colors.secondary_fg),
                Span::from(format!(
                    "https://github.com/pltx/tui/blob/main/docs/{}.md",
                    document.filename
                )),
            ])
            .fg(colors.primary),
        ])
        .block(
            Block::new()
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(colors.border)),
        );

        let highlighted_content = self
            .highlighted_content
            .clone()
            .expect("failed to unwrap highlighted document content")
            .split('\n')
            .collect::<Vec<&str>>()[self.from_top..content_layout.height as usize + self.from_top]
            .join("\n")
            .into_text()
            .expect("failed to convert ansi to text");

        let content =
            Paragraph::new(highlighted_content).block(Block::new().padding(Padding::horizontal(1)));

        frame.render_widget(frontmatter, frontmatter_layout);
        frame.render_widget(content, scrollable_content_layout);
    }

    fn render_selection(&self, app: &App, frame: &mut Frame, area: Rect) {
        let colors = &app.config.colors;

        let table = DOCUMENTS
            .iter()
            .enumerate()
            .map(|(i, d)| {
                Paragraph::new(format!(" {} ", d.frontmatter.title)).bg(
                    if self.selected_page == i {
                        colors.input_focus_bg
                    } else {
                        colors.bg
                    },
                )
            })
            .collect::<Vec<Paragraph>>();

        let area = centered_rect((100, false), (20, false), area);
        let card = Card::new("Select a Help Page", area);
        card.render(frame, app, area, false);

        self.scrollable.render(frame, card.child_layout(), table);
    }
}
