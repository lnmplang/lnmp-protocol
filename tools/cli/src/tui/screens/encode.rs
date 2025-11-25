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
            "🔐 Encode to Binary",
            Style::default().fg(Color::Magenta),
        )),
        Line::from(""),
        Line::from("Convert LNMP text to binary format"),
        Line::from(""),
        Line::from(Span::styled(
            "Coming soon...",
            Style::default().fg(Color::Gray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Encode Screen");
    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
