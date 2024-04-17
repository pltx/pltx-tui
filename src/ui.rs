use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::*, Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{
    popups,
    state::{Mode, Pane},
    utils::RenderPopup,
    App, Popup,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let colors = &app.config.colors.clone();

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
        .style(Style::new().fg(colors.title_bar_fg).bg(colors.title_bar_bg));
    frame.render_widget(title_bar, layout[0]);

    // Navigation and editor layout
    let panes_layout = Layout::default()
        .constraints([Constraint::Length(30), Constraint::Min(1)])
        .direction(Direction::Horizontal)
        .split(layout[1]);

    // Navigation
    let navigation_options = &app
        .screen_list
        .iter()
        .map(|s| {
            let mut style = Style::new();
            if s.0 == app.state.screen {
                style = style.fg(colors.active).bold()
            } else {
                style = style.fg(colors.secondary)
            };
            Line::from(s.1).style(style)
        })
        .collect::<Vec<Line>>();
    let navigation_pane = Paragraph::new(navigation_options.clone()).block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(match (&app.state.mode, &app.state.pane) {
                (Mode::Navigation, Pane::Navigation) => colors.primary,
                _ => colors.border,
            }))
            .padding(Padding::symmetric(1, 0))
            .bg(colors.bg),
    );
    frame.render_widget(navigation_pane, panes_layout[0]);

    // Screen content
    let screen_pane = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(match (&app.state.mode, &app.state.pane) {
            (Mode::Navigation, Pane::Screen) => colors.primary,
            _ => colors.border,
        }))
        .padding(Padding::horizontal(1))
        .bg(colors.bg);
    frame.render_widget(screen_pane, panes_layout[1]);

    let screen_index = app
        .screen_list
        .iter()
        .position(|s| s.0 == app.state.screen)
        .unwrap();
    let screen_layout = Layout::default()
        .vertical_margin(1)
        .horizontal_margin(2)
        .constraints([Constraint::Min(1)])
        .split(panes_layout[1]);
    app.screen_list[screen_index].2(frame, app, screen_layout[0]);

    // Popup
    if app.state.mode == Mode::Popup {
        match app.state.popup {
            Popup::Help => popups::Help::init().render(frame, app),
            Popup::None => {}
        }
    }

    // Status bar
    let mode = app.get_mode();
    let status_bar_content = vec![Line::from(vec![
        Span::from(format!(" {} ", mode.0.to_uppercase()))
            .bold()
            .fg(mode.1)
            .bg(mode.2),
        Span::from("").fg(mode.2),
    ])];
    let status_bar = Paragraph::new(status_bar_content)
        .alignment(Alignment::Left)
        .style(
            Style::new()
                .fg(colors.status_bar_fg)
                .bg(colors.status_bar_bg),
        );
    frame.render_widget(status_bar, layout[2]);
}
