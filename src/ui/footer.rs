//! Footer rendering

use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub fn draw_footer(f: &mut Frame, area: Rect, theme: &Theme, running: bool) {
    let text = if running {
        format!("{} Please wait for the result.", theme.copy.running_message)
    } else {
        format!(
            "{} · {} · {} · {}",
            theme.copy.footer_nav,
            theme.copy.footer_select,
            theme.copy.footer_back,
            theme.copy.footer_quit,
        )
    };

    let footer = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(theme.footer)
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
