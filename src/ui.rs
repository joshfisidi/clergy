use std::io::{self, Stdout};
use std::process::Command;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use crate::about;
use crate::safety;
use crate::model::PurgeData;

// ─────────────────────────────────────────────────────────────
// UI STATE
// ─────────────────────────────────────────────────────────────

enum UiState {
    Dashboard,
    ConfirmPurge,
    Running,
    Result(PurgeData),
    Explain,
    Status,
    About,
    Error(String),
}

enum ConfirmChoice {
    Confirm,
    Cancel,
}

struct MenuState {
    selected: usize,
}

const MENU_ITEMS: &[&str] = &[
    "Run system purge",
    "View last purge result",
    "Explain what will be purged",
    "System status overview",
    "About CLERGY",
    "Quit",
];

// ─────────────────────────────────────────────────────────────
// PUBLIC ENTRY POINTS
// ─────────────────────────────────────────────────────────────

fn ensure_sudo() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("sudo")
        .arg("-v")
        .status()?;

    if !status.success() {
        return Err("Administrator privileges required".into());
    }
    Ok(())
}

pub fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    // Authenticate BEFORE TUI (prompts in normal terminal)
    ensure_sudo()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = UiState::Dashboard;
    let mut menu = MenuState { selected: 0 };
    let mut confirm_choice = ConfirmChoice::Confirm;

    let result = ui_loop(&mut terminal, &mut state, &mut menu, &mut confirm_choice);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Display purge results in TUI (called from `clergy purge`)
pub fn render_purge_ui(data: &PurgeData) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let layout = dashboard_layout(f.area());
            draw_logo(f, layout.logo);
            draw_menu(f, layout.menu, 0);
            draw_stats(f, layout.stats);
            draw_footer(f, layout.footer);
            draw_result(f, layout.main, data);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// MAIN LOOP
// ─────────────────────────────────────────────────────────────

fn ui_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut UiState,
    menu: &mut MenuState,
    confirm_choice: &mut ConfirmChoice,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| {
            let layout = dashboard_layout(f.area());

            draw_logo(f, layout.logo);
            draw_menu(f, layout.menu, menu.selected);
            draw_stats(f, layout.stats);
            draw_footer(f, layout.footer);

            match state {
                UiState::Dashboard => draw_welcome(f, layout.main),
                UiState::ConfirmPurge => draw_confirm_purge(f, layout.main, confirm_choice),
                UiState::Running => draw_running(f, layout.main),
                UiState::Result(data) => draw_result(f, layout.main, data),
                UiState::Explain => draw_explain(f, layout.main),
                UiState::Status => draw_welcome(f, layout.main), // placeholder
                UiState::About => draw_about(f, layout.main),
                UiState::Error(msg) => draw_error(f, layout.main, msg),
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle confirmation screen inputs first
                if matches!(state, UiState::ConfirmPurge) {
                    match key.code {
                        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                            *confirm_choice = match confirm_choice {
                                ConfirmChoice::Confirm => ConfirmChoice::Cancel,
                                ConfirmChoice::Cancel => ConfirmChoice::Confirm,
                            };
                        }
                        KeyCode::Enter => {
                            match confirm_choice {
                                ConfirmChoice::Confirm => {
                                    match safety::can_run_purge() {
                                        Ok(()) => {
                                            *state = UiState::Running;

                                            match crate::shell::run_purge() {
                                                Ok(data) => {
                                                    safety::mark_purge_run();
                                                    *state = UiState::Result(data);
                                                }
                                                Err(e) => {
                                                    *state = UiState::Error(e.to_string());
                                                }
                                            }
                                        }
                                        Err(remaining) => {
                                            *state = UiState::Error(format!(
                                                "Purge recently performed.\nPlease wait {} seconds before running again.",
                                                remaining.as_secs()
                                            ));
                                        }
                                    }
                                }
                                ConfirmChoice::Cancel => {
                                    *state = UiState::Dashboard;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            *state = UiState::Dashboard;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Normal menu handling
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        menu.selected = menu.selected.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        menu.selected = (menu.selected + 1).min(MENU_ITEMS.len() - 1);
                    }
                    KeyCode::Enter => match menu.selected {
                        // Run system purge -> show confirmation
                        0 => {
                            *state = UiState::ConfirmPurge;
                            *confirm_choice = ConfirmChoice::Confirm;
                        }
                        // View last purge result
                        1 => {
                            *state = UiState::Dashboard;
                        }
                        // Explain purge
                        2 => {
                            *state = UiState::Explain;
                        }
                        // System status
                        3 => {
                            *state = UiState::Status;
                        }
                        // About
                        4 => {
                            *state = UiState::About;
                        }
                        // Quit
                        5 => return Ok(()),
                        _ => {}
                    },
                    KeyCode::Esc | KeyCode::Backspace => {
                        *state = UiState::Dashboard;
                    }
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// DASHBOARD LAYOUT
// ─────────────────────────────────────────────────────────────

struct DashboardLayout {
    logo: Rect,
    menu: Rect,
    main: Rect,
    stats: [Rect; 3],
    footer: Rect,
}

fn dashboard_layout(area: Rect) -> DashboardLayout {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(7),
            Constraint::Length(1),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),
            Constraint::Min(20),
        ])
        .split(vertical[0]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(5),
        ])
        .split(top[0]);

    let stats = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(vertical[1]);

    DashboardLayout {
        logo: left[0],
        menu: left[1],
        main: top[1],
        stats: [stats[0], stats[1], stats[2]],
        footer: vertical[2],
    }
}

// ─────────────────────────────────────────────────────────────
// DRAW COMPONENTS
// ─────────────────────────────────────────────────────────────

fn draw_logo(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .title("CLERGY")
        .borders(Borders::ALL);
    f.render_widget(block, area);
}

fn draw_menu(f: &mut ratatui::Frame, area: Rect, selected: usize) {
    let mut lines = vec![Line::from("")];
    for (i, label) in MENU_ITEMS.iter().enumerate() {
        let line = if i == selected {
            Line::from(Span::styled(
                format!(" ❯ {}", label),
                Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ))
        } else {
            Line::from(format!("   {}", label))
        };
        lines.push(line);
    }

    let menu = Paragraph::new(lines)
        .block(Block::default().title("Menu").borders(Borders::ALL));

    f.render_widget(menu, area);
}

fn draw_welcome(f: &mut ratatui::Frame, area: Rect) {
    let text = Paragraph::new("Select an action from the menu")
        .block(Block::default().title("Dashboard").borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn draw_running(f: &mut ratatui::Frame, area: Rect) {
    let text = Paragraph::new(vec![
        Line::from("Administrator permission is required."),
        Line::from("You may be prompted for your password."),
    ])
    .block(Block::default().title("Running purge…").borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn draw_result(f: &mut ratatui::Frame, area: Rect, data: &PurgeData) {
    let body = Paragraph::new(vec![
        Line::from(format!("Host: {}", data.host)),
        Line::from(format!("User: {}", data.user)),
        Line::from(format!("Duration: {}s", data.duration_seconds)),
        Line::from(""),
        Line::from(format!("DNS flushed: {}", yes_no(data.dns_flushed))),
        Line::from(format!("Snapshots thinned: {}", yes_no(data.snapshots_thinned))),
    ])
    .block(Block::default().title("Result").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    f.render_widget(body, area);
}

fn draw_error(f: &mut ratatui::Frame, area: Rect, msg: &str) {
    let text = Paragraph::new(msg)
        .block(Block::default().title("Error").borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn draw_explain(f: &mut ratatui::Frame, area: Rect) {
    let text = Paragraph::new(vec![
        Line::from("CLERGY performs the following actions:"),
        Line::from(""),
        Line::from("• Flushes macOS DNS caches"),
        Line::from("• Thins Time Machine local snapshots"),
        Line::from("• Does NOT delete user files"),
        Line::from("• Does NOT modify system configuration"),
    ])
    .block(Block::default().title("What CLERGY Does").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    f.render_widget(text, area);
}

fn draw_about(f: &mut ratatui::Frame, area: Rect) {
    let text = Paragraph::new(about::about_lines().join("\n"))
        .block(Block::default().title("About").borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn draw_stats(f: &mut ratatui::Frame, areas: [Rect; 3]) {
    let titles = ["CPU", "Memory", "Disk"];
    for (i, area) in areas.iter().enumerate() {
        let block = Block::default()
            .title(titles[i])
            .borders(Borders::ALL);
        f.render_widget(block, *area);
    }
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect) {
    let text = about::about_lines().join(" · ");

    let footer = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::DIM),
        )
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

fn draw_confirm_purge(f: &mut ratatui::Frame, area: Rect, choice: &ConfirmChoice) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    let message = Paragraph::new(vec![
        Line::from("You are about to perform a system purge."),
        Line::from(""),
        Line::from("This will:"),
        Line::from("• Flush DNS caches"),
        Line::from("• Thin local Time Machine snapshots"),
        Line::from(""),
        Line::from("Administrator privileges are required."),
    ])
    .block(Block::default().title("Confirm purge").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    f.render_widget(message, vertical[0]);

    let buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(vertical[1]);

    draw_button(f, buttons[0], "Confirm", matches!(choice, ConfirmChoice::Confirm));
    draw_button(f, buttons[1], "Cancel", matches!(choice, ConfirmChoice::Cancel));
}

fn draw_button(f: &mut ratatui::Frame, area: Rect, label: &str, active: bool) {
    let style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .style(style);

    let button = Paragraph::new(label)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(button, area);
}

// ─────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────

fn yes_no(v: bool) -> &'static str {
    if v { "yes" } else { "no" }
}
