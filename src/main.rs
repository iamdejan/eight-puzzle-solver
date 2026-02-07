#![deny(unused_variables)]
#![deny(unused_imports)]

mod a_star;
mod board;

use anyhow::Result;
use board::Board;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
};
use std::{
    io,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

enum AppMode {
    Input,
    Searching,
    Result,
}

struct App {
    mode: AppMode,
    // Input state
    input_board: Board,
    cursor_pos: (usize, usize), // (row, col)
    error_msg: Option<String>,

    // Search state
    rx_result: Option<Receiver<Option<Vec<Board>>>>,
    spinner_idx: usize,

    // Result state
    solution_path: Vec<Board>,
    current_step: usize,
}

impl App {
    fn new() -> App {
        App {
            mode: AppMode::Input,
            input_board: Board::default(),
            cursor_pos: (0, 0),
            error_msg: None,
            rx_result: None,
            spinner_idx: 0,
            solution_path: Vec::new(),
            current_step: 0,
        }
    }

    fn on_tick(&mut self) {
        // Update spinner animation
        if let AppMode::Searching = self.mode {
            self.spinner_idx = (self.spinner_idx + 1) % 4;

            // Check if thread finished
            if let Some(rx) = &self.rx_result {
                match rx.try_recv() {
                    Ok(result) => {
                        match result {
                            Some(path) => {
                                self.solution_path = path;
                                self.current_step = 0;
                                self.mode = AppMode::Result;
                            }
                            None => {
                                self.error_msg =
                                    Some("No solution found for this configuration.".to_string());
                                self.mode = AppMode::Input;
                            }
                        }
                        self.rx_result = None;
                    }
                    Err(TryRecvError::Empty) => {} // Still working
                    Err(TryRecvError::Disconnected) => {
                        self.error_msg = Some("Search thread panicked.".to_string());
                        self.mode = AppMode::Input;
                        self.rx_result = None;
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App
    let mut app = App::new();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)?
            && let Event::Key(key) = event::read()?
        {
            match app.mode {
                AppMode::Input => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Left => {
                            if app.cursor_pos.1 > 0 {
                                app.cursor_pos.1 -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if app.cursor_pos.1 < 2 {
                                app.cursor_pos.1 += 1;
                            }
                        }
                        KeyCode::Up => {
                            if app.cursor_pos.0 > 0 {
                                app.cursor_pos.0 -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.cursor_pos.0 < 2 {
                                app.cursor_pos.0 += 1;
                            }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            let digit = c.to_digit(10).unwrap();
                            if (1..=8).contains(&digit) {
                                app.input_board.b[app.cursor_pos.0][app.cursor_pos.1] =
                                    Some(digit as i64);
                            } else if digit == 0 {
                                app.input_board.b[app.cursor_pos.0][app.cursor_pos.1] = None;
                            }
                        }
                        KeyCode::Backspace | KeyCode::Delete | KeyCode::Char(' ') => {
                            app.input_board.b[app.cursor_pos.0][app.cursor_pos.1] = None;
                        }
                        KeyCode::Enter => {
                            // Validate and Start Search
                            if app.input_board.is_valid() {
                                app.mode = AppMode::Searching;
                                app.error_msg = None;

                                let board_clone = app.input_board;
                                let (tx, rx) = mpsc::channel();
                                app.rx_result = Some(rx);

                                // Spawn search thread
                                thread::spawn(move || {
                                    let result = a_star::search(board_clone);
                                    tx.send(result).unwrap();
                                });
                            } else {
                                app.error_msg = Some(
                                    "Invalid Board: Must contain 1-8 unique & 1 empty.".to_string(),
                                );
                            }
                        }
                        _ => {}
                    }
                }
                AppMode::Searching => {
                    // Consume keys but do nothing, or allow 'q' to abort
                    if let KeyCode::Char('q') = key.code {
                        // Simple abort by resetting app (thread keeps running detached but ignored)
                        app.mode = AppMode::Input;
                        app.rx_result = None;
                    }
                }
                AppMode::Result => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.mode = AppMode::Input; // Return to editor
                        }
                        KeyCode::Left => {
                            if app.current_step > 0 {
                                app.current_step -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if app.current_step < app.solution_path.len() - 1 {
                                app.current_step += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer (Instructions)
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let title = Paragraph::new("Rust A* 8-Puzzle Solver")
        .block(title_block)
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Footer
    let footer_text = match app.mode {
        AppMode::Input => "Arrows: Move | 0-8: Fill | Space: Empty | Enter: Solve | q: Quit",
        AppMode::Searching => "Calculating... Please wait...",
        AppMode::Result => "Left/Right: Prev/Next Step | q: New Puzzle",
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);

    // Main Content
    let content_area = chunks[1];

    match app.mode {
        AppMode::Input => draw_input(f, app, content_area),
        AppMode::Searching => draw_searching(f, app, content_area),
        AppMode::Result => draw_result(f, app, content_area),
    }
}

fn draw_board(
    f: &mut Frame,
    board: &Board,
    area: ratatui::layout::Rect,
    highlight_pos: Option<(usize, usize)>,
) {
    // Create a 3x3 layout centered in the area
    let layout_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    for r in 0..3 {
        let layout_h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(7),
                    Constraint::Length(7),
                    Constraint::Length(7),
                ]
                .as_ref(),
            )
            .split(layout_v[r]);

        for c in 0..3 {
            let cell_value = match board.b[r][c] {
                Some(v) => v.to_string(),
                None => " ".to_string(),
            };

            let mut style = Style::default().fg(Color::White);
            let mut border_style = Style::default();

            // Highlight cursor if in Input mode
            if let Some((hr, hc)) = highlight_pos
                && r == hr
                && c == hc
            {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                border_style = border_style.fg(Color::Yellow);
            }

            // Highlight 'None' (empty tile) distinctively in result view
            if board.b[r][c].is_none() {
                style = style.bg(Color::DarkGray);
            }

            let p = Paragraph::new(cell_value)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .alignment(Alignment::Center)
                .style(style);

            f.render_widget(p, layout_h[c]);
        }
    }
}

fn draw_input(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)].as_ref())
        .split(area);

    // Center the board area
    let board_area_centered = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Length(21),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(chunks[0])[1];

    draw_board(
        f,
        &app.input_board,
        board_area_centered,
        Some(app.cursor_pos),
    );

    if let Some(err) = &app.error_msg {
        let err_widget = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(err_widget, chunks[1]);
    }
}

fn draw_searching(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let spinners = ["|", "/", "-", "\\"];
    let spinner = spinners[app.spinner_idx];

    let text = format!("Solving... {}", spinner);

    let p = Paragraph::new(text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    // Center vertically
    let v_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Length(3),
                Constraint::Percentage(45),
            ]
            .as_ref(),
        )
        .split(area);

    f.render_widget(p, v_layout[1]);

    // Render a "Progress Bar" (Indeterminate)
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Searching"))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(100) // Since we can't track A* progress inside the function, we just show a full bar or pulsing
        .label("Computing Path...");

    f.render_widget(gauge, v_layout[1]);
}

fn draw_result(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let board = &app.solution_path[app.current_step];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(10),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let step_info = format!(
        "Step {} / {}",
        app.current_step + 1,
        app.solution_path.len()
    );
    let info_p = Paragraph::new(step_info)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(info_p, chunks[0]);

    let board_area_centered = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Length(21),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(chunks[1])[1];

    draw_board(f, board, board_area_centered, None);
}
