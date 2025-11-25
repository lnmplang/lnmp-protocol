use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::state::AppState;

pub fn render(f: &mut Frame, area: Rect, _state: &AppState) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "✓ Validate LNMP",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("Validate LNMP file syntax and structure"),
        Line::from(""),
        Line::from(Span::styled(
            "Coming soon...",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Validate Screen");
    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
