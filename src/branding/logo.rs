use ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

/// Height of the logo in terminal rows
pub const LOGO_HEIGHT: u16 = 7;

/// Width of the logo in terminal columns (measured from ASCII art)
#[allow(dead_code)]
pub const LOGO_WIDTH: u16 = 89;

/// Raw logo strings
const LOGO_ROWS: &[&str] = &[
    " ░▒▓██████▓▒░░▒▓█▓▒░      ░▒▓████████▓▒░▒▓███████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░░▒▓█▓▒░ ",
    "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░ ",
    "░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░ ",
    "░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓██████▓▒░ ░▒▓███████▓▒░░▒▓█▓▒▒▓███▓▒░░▒▓██████▓▒░  ",
    "░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░  ░▒▓█▓▒░     ",
    "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░  ░▒▓█▓▒░     ",
    " ░▒▓██████▓▒░░▒▓████████▓▒░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░░▒▓██████▓▒░   ░▒▓█▓▒░     ",
];

/// Get all logo rows as a vector (unstyled, for backwards compat)
pub fn clergy_logo_rows() -> Vec<Line<'static>> {
    LOGO_ROWS.iter().map(|s| Line::from(*s)).collect()
}

/// Get logo rows with styling applied
pub fn clergy_logo_rows_styled(style: Style) -> Vec<Line<'static>> {
    LOGO_ROWS
        .iter()
        .map(|s| Line::from(Span::styled(*s, style)))
        .collect()
}

/// Render partial logo (for animation) — unstyled
pub fn clergy_logo_partial(rows: usize) -> Paragraph<'static> {
    let all = clergy_logo_rows();
    let visible: Vec<Line<'static>> = all.into_iter().take(rows).collect();

    Paragraph::new(Text::from(visible))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center)
}

/// Render partial logo with theme styling
pub fn clergy_logo_partial_styled(rows: usize, style: Style) -> Paragraph<'static> {
    let all = clergy_logo_rows_styled(style);
    let visible: Vec<Line<'static>> = all.into_iter().take(rows).collect();

    Paragraph::new(Text::from(visible))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center)
}

/// Render full logo (static)
pub fn clergy_logo() -> Paragraph<'static> {
    clergy_logo_partial(LOGO_HEIGHT as usize)
}
