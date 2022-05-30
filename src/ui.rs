use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use unicode_width::UnicodeWidthStr;

use crate::app::{App, Focus, InputMode};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    draw_input(f, app, chunks[0]);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(30), Constraint::Percentage(80)].as_ref())
            .direction(Direction::Horizontal)
            .split(chunks[1]);
    }
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::raw("Enter station:"));

    let (input_normal_mode_message, input_normal_mode_style) = (
        vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to exit, ", Style::default().fg(Color::DarkGray)),
            Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to insert.", Style::default().fg(Color::DarkGray)),
        ],
        Style::default().add_modifier(Modifier::RAPID_BLINK),
    );

    let mut text = Text::from(Spans::from(input_normal_mode_message));
    text.patch_style(input_normal_mode_style);

    let input = match app.input_mode {
        InputMode::Insert => Paragraph::new(app.input.as_ref())
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .block(block),
        InputMode::Normal => Paragraph::new(text).style(Style::default()).block(block),
    };

    f.render_widget(input, area);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Insert => f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1),
    }
}
