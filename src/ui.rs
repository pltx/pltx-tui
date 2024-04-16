use std::str::FromStr;

use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{block::*, Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame) {
    let primary_color = Color::from_str("#AF5FFF").unwrap();

    // Root layout
    let layout = Layout::default()
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.size());

    // Title bar
    let title_bar_content = vec![Line::from(
        vec![Span::from(" Privacy Life Tracker ").bold()],
    )];
    let title_bar = Paragraph::new(title_bar_content)
        .alignment(Alignment::Center)
        .style(Style::new().bg(primary_color));
    frame.render_widget(title_bar, layout[0]);

    // Navigation and editor layout
    let window_layout = Layout::default()
        .constraints([Constraint::Length(30), Constraint::Min(1)])
        .direction(Direction::Horizontal)
        .split(layout[1]);

    // Navigation
    let navigation_options = vec![
        Line::styled(" Option #1 ", Style::new().bold().fg(Color::Blue)),
        Line::styled(" Option #2 ", Style::new().bold().fg(Color::Gray)),
    ];
    let navigation = Paragraph::new(navigation_options).block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(primary_color))
            .padding(Padding::symmetric(1, 0))
            .style(Style::new().on_black()),
    );
    frame.render_widget(navigation, window_layout[0]);

    // Main content
    let main_content = Paragraph::new("Main content").block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().dark_gray())
            .padding(Padding::horizontal(1))
            .style(Style::new().on_black()),
    );
    frame.render_widget(main_content, window_layout[1]);

    // Status bar
    let status_bar_content = vec![Line::from(vec![Span::from("Status bar")])];
    let status_bar = Paragraph::new(status_bar_content)
        .alignment(Alignment::Center)
        .style(Style::new().on_dark_gray());
    frame.render_widget(status_bar, layout[2]);
}
