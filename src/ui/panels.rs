//! Main panel rendering (menu, about, settings, result, etc.)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::ConfirmChoice;
use crate::branding::{about_text, attribution, clergy_logo_partial_styled, LOGO_HEIGHT, VERSION};
use crate::config::Settings;
use crate::system::Metrics;
use crate::theme::Theme;

/// Menu item labels
pub const MENU_ITEMS: &[&str] = &[
    "Purge",
    "Last Session",
    "How it works",
    "System status",
    "About",
    "Settings",
    "Quit",
];

/// Centered column width — all panels share this anchor
const MAIN_COLUMN_WIDTH: u16 = 92;

/// Compute a centered panel rect (same dimensions as main menu)
pub(super) fn centered_panel(area: Rect) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(MAIN_COLUMN_WIDTH) / 2,
        y: area.y,
        width: MAIN_COLUMN_WIDTH.min(area.width),
        height: area.height,
    }
}

/// Standard panel layout: logo at top, content below
/// Returns (logo_area, content_area)
fn panel_with_logo(area: Rect) -> (Rect, Rect) {
    let panel = centered_panel(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(LOGO_HEIGHT + 2), // logo with breathing room
            Constraint::Length(1),               // spacer
            Constraint::Min(4),                  // content
        ])
        .split(panel);
    (chunks[0], chunks[2])
}

/// Draw the dashboard main content: centered logo + centered menu
pub fn draw_menu_column(
    f: &mut Frame,
    area: Rect,
    selected: usize,
    logo_rows: usize,
    theme: &Theme,
) {
    // First: create a centered column for logo + menu
    let centered = Rect {
        x: area.x + area.width.saturating_sub(MAIN_COLUMN_WIDTH) / 2,
        y: area.y,
        width: MAIN_COLUMN_WIDTH.min(area.width),
        height: area.height,
    };

    // Vertical split within the centered column
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(LOGO_HEIGHT + 2), // logo with breathing room
            Constraint::Length(1),               // spacer row
            Constraint::Min(10),                 // menu panel
        ])
        .split(centered);

    // Logo: centered within its chunk (already in centered column)
    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), chunks[0]);

    // chunks[1] = spacer (intentionally empty)

    // Menu: fills the centered column (same width as logo area)
    draw_menu_panel(f, chunks[2], selected, theme);
}

/// Draw menu options inside a single bordered panel (no logo)
fn draw_menu_panel(f: &mut Frame, area: Rect, selected: usize, theme: &Theme) {
    let block = Block::default()
        .title(format!(" {} ", theme.copy.menu_title))
        .borders(Borders::ALL)
        .border_style(theme.panel_border_focused)
        .style(theme.panel_bg);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::with_capacity(MENU_ITEMS.len() * 2);

    for (i, label) in MENU_ITEMS.iter().enumerate() {
        let (prefix, style) = if i == selected {
            (theme.copy.menu_prefix_selected, theme.menu_selected)
        } else {
            (theme.copy.menu_prefix_normal, theme.menu_normal)
        };
        lines.push(Line::from(Span::styled(format!("{prefix}{label}"), style)));
        lines.push(Line::from(""));
    }

    if !lines.is_empty() {
        lines.pop();
    }

    let menu = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(menu, inner);
}

pub fn draw_running(f: &mut Frame, area: Rect, logo_rows: usize, theme: &Theme) {
    let (logo_area, content_area) = panel_with_logo(area);

    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), logo_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(theme.copy.running_message),
        Line::from(""),
        Line::from("Please wait."),
    ])
    .block(Block::default().title("Running").borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(text, content_area);
}

pub fn draw_error(f: &mut Frame, area: Rect, msg: &str, logo_rows: usize, theme: &Theme) {
    let (logo_area, content_area) = panel_with_logo(area);

    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), logo_area);

    let text = Paragraph::new(msg)
        .block(Block::default().title(theme.copy.error_title).borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(text, content_area);
}

pub fn draw_explain(f: &mut Frame, area: Rect, logo_rows: usize, theme: &Theme) {
    let (logo_area, content_area) = panel_with_logo(area);

    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), logo_area);

    let text = Paragraph::new(vec![
        Line::from("CLERGY performs the following actions:"),
        Line::from(""),
        Line::from("• Flushes macOS DNS caches"),
        Line::from("• Thins Time Machine local snapshots"),
        Line::from("• Does NOT delete user files"),
        Line::from("• Does NOT modify system configuration"),
    ])
    .block(Block::default().title(theme.copy.explain_title).borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    f.render_widget(text, content_area);
}

pub fn draw_status(f: &mut Frame, area: Rect, metrics: &Metrics, logo_rows: usize, theme: &Theme) {
    let (logo_area, content_area) = panel_with_logo(area);

    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), logo_area);

    let swap_total = metrics.swap.used_mb + metrics.swap.free_mb;

    let text = Paragraph::new(vec![
        Line::from("Live System Status"),
        Line::from(""),
        Line::from(format!("CPU: {:.1}% ({} cores)", metrics.cpu.usage, metrics.cpu.cores)),
        Line::from(format!("Memory: {} / {} MB", metrics.mem.used_mb, metrics.mem.total_mb)),
        Line::from(format!("Disk: {} GB free / {} GB total", metrics.disk.free_gb, metrics.disk.total_gb)),
        Line::from(format!("Swap: {} / {} MB", metrics.swap.used_mb, swap_total)),
    ])
    .block(Block::default().title(theme.copy.status_title).borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    f.render_widget(text, content_area);
}

pub fn draw_about(f: &mut Frame, area: Rect, logo_rows: usize, theme: &Theme) {
    let panel = centered_panel(area);

    // Split: logo | about text | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(LOGO_HEIGHT + 2), // logo
            Constraint::Min(6),                  // about text
            Constraint::Length(3),               // footer
        ])
        .split(panel);

    // Render animated logo
    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), chunks[0]);

    // Render about text
    let about_lines: Vec<Line> = about_text()
        .iter()
        .map(|s| Line::from(*s))
        .collect();

    let about_text = Paragraph::new(about_lines)
        .block(Block::default().title(theme.copy.about_title).borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(about_text, chunks[1]);

    // Render footer with version and attribution
    let footer_text = format!(
        "v{} · {} · Press Esc to return",
        VERSION,
        attribution().join(" · ")
    );

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(theme.dim)
        .alignment(Alignment::Center);

    f.render_widget(footer, chunks[2]);
}

pub fn draw_settings(
    f: &mut Frame,
    area: Rect,
    settings: &Settings,
    settings_state: crate::app::SettingsState,
    logo_rows: usize,
    theme: &Theme,
) {
    let (logo_area, content_area) = panel_with_logo(area);

    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), logo_area);

    let block = Block::default()
        .title(theme.copy.settings_title)
        .borders(Borders::ALL)
        .border_style(theme.button_active_border);

    let inner = block.inner(content_area);
    f.render_widget(block, content_area);

    // Split into rows for each setting
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Theme row
            Constraint::Length(2), // Spacer
            Constraint::Length(2), // Purge info (read-only)
            Constraint::Length(2), // Confirm info (read-only)
            Constraint::Min(2),    // Footer
        ])
        .split(inner);

    // ───── Theme row (editable) ─────
    let theme_selected = settings_state.selected == 0;
    let theme_style = if theme_selected {
        theme.menu_selected
    } else {
        theme.menu_normal
    };

    let theme_label = if theme_selected {
        format!("◀  {}  ▶", settings.theme.name())
    } else {
        settings.theme.name().to_string()
    };

    let theme_row = Paragraph::new(vec![
        Line::from(Span::styled("Theme", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(theme_label, theme_style)),
    ])
    .alignment(Alignment::Center);

    f.render_widget(theme_row, rows[0]);

    // ───── Purge info (read-only) ─────
    let confirm_status = if settings.confirm_purge() {
        "enabled"
    } else {
        "disabled"
    };

    let purge_info = Paragraph::new(Line::from(Span::styled(
        format!("Cooldown: {}s", settings.purge.cooldown_secs),
        theme.dim,
    )))
    .alignment(Alignment::Center);

    f.render_widget(purge_info, rows[2]);

    let confirm_info = Paragraph::new(Line::from(Span::styled(
        format!("Confirm: {}", confirm_status),
        theme.dim,
    )))
    .alignment(Alignment::Center);

    f.render_widget(confirm_info, rows[3]);

    // ───── Footer ─────
    let footer = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled("← → to change · Esc to return", theme.dim)),
    ])
    .alignment(Alignment::Center);

    f.render_widget(footer, rows[4]);
}

pub fn draw_confirm_purge(f: &mut Frame, area: Rect, choice: &ConfirmChoice, logo_rows: usize, theme: &Theme) {
    let (logo_area, content_area) = panel_with_logo(area);

    f.render_widget(clergy_logo_partial_styled(logo_rows, theme.header), logo_area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(content_area);

    let message = Paragraph::new(vec![
        Line::from("You are about to perform a system purge."),
        Line::from(""),
        Line::from("This will:"),
        Line::from("• Flush DNS caches"),
        Line::from("• Thin local Time Machine snapshots"),
        Line::from(""),
        Line::from("Administrator privileges are required."),
    ])
    .block(Block::default().title(theme.copy.confirm_title).borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    f.render_widget(message, vertical[0]);

    let buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(vertical[1]);

    super::buttons::draw_button(f, buttons[0], "Confirm", *choice == ConfirmChoice::Confirm, theme);
    super::buttons::draw_button(f, buttons[1], "Cancel", *choice == ConfirmChoice::Cancel, theme);
}
