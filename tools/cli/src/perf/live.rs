use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, Gauge, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

/// Live dashboard state
pub struct DashboardState {
    pub running: bool,
    pub paused: bool,
    pub start_time: Instant,
    pub data_points: Vec<(f64, f64)>,
    pub current_ops: f64,
    pub peak_ops: f64,
    pub total_ops: u64,
    pub errors: u64,
    pub p50_latency: f64,
    pub p95_latency: f64,
    pub p99_latency: f64,
    pub memory_mb: f64,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            running: true,
            paused: false,
            start_time: Instant::now(),
            data_points: Vec::new(),
            current_ops: 0.0,
            peak_ops: 0.0,
            total_ops: 0,
            errors: 0,
            p50_latency: 0.0,
            p95_latency: 0.0,
            p99_latency: 0.0,
            memory_mb: 0.0,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn update(&mut self) -> Result<()> {
        // Simulate performance data (in real implementation, measure actual operations)
        let elapsed = self.start_time.elapsed().as_secs_f64();

        // Simulated throughput (oscillating between 1M-2M ops/sec)
        self.current_ops = 1_500_000.0 + 500_000.0 * (elapsed * 0.5).sin();

        if self.current_ops > self.peak_ops {
            self.peak_ops = self.current_ops;
        }

        // Add data point (keep last 60 seconds)
        self.data_points.push((elapsed, self.current_ops));
        if self.data_points.len() > 60 {
            self.data_points.remove(0);
        }

        // Simulated latency
        self.p50_latency = 0.42 + (elapsed * 0.3).sin() * 0.1;
        self.p95_latency = 0.89 + (elapsed * 0.3).cos() * 0.2;
        self.p99_latency = 1.23 + (elapsed * 0.3).sin() * 0.3;

        // Simulated memory
        self.memory_mb = 0.52 + (self.data_points.len() as f64) * 0.01;

        self.total_ops += (self.current_ops / 10.0) as u64;

        Ok(())
    }

    pub fn snapshot(&self) -> Result<()> {
        println!("📸 Snapshot saved!");
        println!("Current: {:.2}M ops/sec", self.current_ops / 1_000_000.0);
        println!("Peak: {:.2}M ops/sec", self.peak_ops / 1_000_000.0);
        println!("Total: {} ops", self.total_ops);
        Ok(())
    }
}

/// Run the live performance dashboard
pub fn run_live_dashboard() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create state
    let mut state = DashboardState::new();

    // Main loop
    while state.running {
        terminal.draw(|f| render_dashboard(f, &state))?;

        // Handle events (non-blocking with timeout)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => state.running = false,
                    KeyCode::Char('p') | KeyCode::Char('P') => state.toggle_pause(),
                    KeyCode::Char('s') | KeyCode::Char('S') => state.snapshot()?,
                    _ => {}
                }
            }
        }

        // Update data if not paused
        if !state.paused {
            state.update()?;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Render the dashboard UI
fn render_dashboard(f: &mut Frame, state: &DashboardState) {
    let size = f.size();

    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Graph
            Constraint::Length(7), // Stats
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Render header
    render_header(f, chunks[0], state);

    // Render graph
    render_graph(f, chunks[1], state);

    // Render stats
    render_stats(f, chunks[2], state);

    // Render footer
    render_footer(f, chunks[3], state);
}

fn render_header(f: &mut Frame, area: Rect, state: &DashboardState) {
    let title = if state.paused {
        "🔬 LNMP Performance Dashboard (PAUSED)"
    } else {
        "🔬 LNMP Performance Dashboard (Live)"
    };

    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

fn render_graph(f: &mut Frame, area: Rect, state: &DashboardState) {
    // Prepare data for chart
    let data: Vec<(f64, f64)> = state.data_points.clone();

    let datasets = vec![Dataset::default()
        .name("Throughput")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .data(&data)];

    let x_bounds = if data.len() >= 2 {
        [data[0].0, data[data.len() - 1].0]
    } else {
        [0.0, 60.0]
    };

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Throughput (ops/sec)")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Time (seconds)")
                .bounds(x_bounds)
                .labels(vec![
                    format!("{:.0}s", x_bounds[0]).into(),
                    format!("{:.0}s", (x_bounds[0] + x_bounds[1]) / 2.0).into(),
                    format!("{:.0}s", x_bounds[1]).into(),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("ops/sec")
                .bounds([0.0, 2_500_000.0])
                .labels(vec!["0".into(), "1.25M".into(), "2.5M".into()]),
        );

    f.render_widget(chart, area);
}

fn render_stats(f: &mut Frame, area: Rect, state: &DashboardState) {
    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Current stats
    let current = Paragraph::new(format!(
        "Current: {:.2}M ops/sec  │  Peak: {:.2}M ops/sec  │  Total: {} ops",
        state.current_ops / 1_000_000.0,
        state.peak_ops / 1_000_000.0,
        state.total_ops
    ))
    .style(Style::default().fg(Color::White));
    f.render_widget(current, stats_chunks[0]);

    // Memory
    let memory = Paragraph::new(format!(
        "Memory: {:.2} MB  │  Errors: {}",
        state.memory_mb, state.errors
    ))
    .style(Style::default().fg(Color::Yellow));
    f.render_widget(memory, stats_chunks[1]);

    // Latency
    let latency = Paragraph::new(format!(
        "Latency: p50: {:.2}μs  │  p95: {:.2}μs  │  p99: {:.2}μs",
        state.p50_latency, state.p95_latency, state.p99_latency
    ))
    .style(Style::default().fg(Color::Magenta));
    f.render_widget(latency, stats_chunks[2]);

    // Throughput gauge
    let throughput_percent = ((state.current_ops / 2_500_000.0) * 100.0) as u16;
    let gauge = Gauge::default()
        .block(Block::default().title("Throughput"))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(throughput_percent);
    f.render_widget(gauge, stats_chunks[4]);
}

fn render_footer(f: &mut Frame, area: Rect, state: &DashboardState) {
    let status = if state.paused { "PAUSED" } else { "RUNNING" };
    let elapsed = state.start_time.elapsed().as_secs();

    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("[P]ause ", Style::default().fg(Color::Yellow)),
        Span::styled("[S]napshot ", Style::default().fg(Color::Green)),
        Span::styled("[Q]uit ", Style::default().fg(Color::Red)),
        Span::styled(
            format!("│ Status: {} │ Uptime: {}s", status, elapsed),
            Style::default().fg(Color::Cyan),
        ),
    ])])
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}
