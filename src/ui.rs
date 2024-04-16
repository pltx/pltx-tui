use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::*, Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::config::Config;

pub fn render(frame: &mut Frame, config: &Config) {
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
        .style(Style::new().bg(config.colors.primary));
    frame.render_widget(title_bar, layout[0]);

    // Navigation and editor layout
    let window_layout = Layout::default()
        .constraints([Constraint::Length(30), Constraint::Min(1)])
        .direction(Direction::Horizontal)
        .split(layout[1]);

    // Navigation
    let navigation_options = vec![
        Line::styled(" Option #1 ", Style::new().bold().fg(config.colors.primary)),
        Line::styled(
            " Option #2 ",
            Style::new().bold().fg(config.colors.secondary),
        ),
    ];
    let navigation = Paragraph::new(navigation_options).block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(config.colors.primary))
            .padding(Padding::symmetric(1, 0))
            .style(Style::new().on_black()),
    );
    frame.render_widget(navigation, window_layout[0]);

    // Main content
    let main_content = Paragraph::new("Main content").block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(config.colors.border))
            .padding(Padding::horizontal(1))
            .style(Style::new().on_black()),
    );
    frame.render_widget(main_content, window_layout[1]);

    // Status bar
    let status_bar_content = vec![Line::from(vec![Span::from("Status bar")])];
    let status_bar = Paragraph::new(status_bar_content)
        .alignment(Alignment::Center)
        .style(
            Style::new()
                .fg(config.colors.status_bar_text)
                .bg(config.colors.status_bar_bg),
        );
    frame.render_widget(status_bar, layout[2]);
}
