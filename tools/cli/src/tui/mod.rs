use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

pub mod screens;
pub mod state;

use state::AppState;

pub struct App {
    state: AppState,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
            should_quit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run app loop
        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            eprintln!("Error: {:?}", err);
        }

        Ok(())
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('1') => self.state.set_screen(state::Screen::Convert),
                        KeyCode::Char('2') => self.state.set_screen(state::Screen::Encode),
                        KeyCode::Char('3') => self.state.set_screen(state::Screen::Decode),
                        KeyCode::Char('4') => self.state.set_screen(state::Screen::Validate),
                        _ => {}
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // Header
        let header = Paragraph::new("🚀 LNMP CLI Manager")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Content
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        // Left sidebar - Menu
        let menu_items = vec![
            ListItem::new("1. Convert JSON ↔ LNMP"),
            ListItem::new("2. Encode to Binary"),
            ListItem::new("3. Decode from Binary"),
            ListItem::new("4. Validate LNMP"),
        ];
        let menu = List::new(menu_items)
            .block(Block::default().borders(Borders::ALL).title("Menu"))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(menu, content_chunks[0]);

        // Right panel - Active screen
        match self.state.current_screen() {
            state::Screen::Convert => screens::convert::render(f, content_chunks[1], &self.state),
            state::Screen::Encode => screens::encode::render(f, content_chunks[1], &self.state),
            state::Screen::Decode => screens::decode::render(f, content_chunks[1], &self.state),
            state::Screen::Validate => screens::validate::render(f, content_chunks[1], &self.state),
        }

        // Footer
        let footer = Paragraph::new(vec![Line::from(vec![
            Span::raw("Press "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" to quit | "),
            Span::styled("1-4", Style::default().fg(Color::Yellow)),
            Span::raw(" to switch screens"),
        ])])
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
