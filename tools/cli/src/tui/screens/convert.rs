use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::AppState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "📂 Convert JSON ↔ LNMP",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("Source File: "),
            Span::styled(&state.source_file, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Target File: "),
            Span::styled(&state.target_file, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Mapping (optional): "),
            Span::styled(
                state.mapping_file.as_deref().unwrap_or("None"),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[ Convert ]",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(&state.status_message, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Coming soon: File browser, real-time conversion",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Convert Screen");

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
