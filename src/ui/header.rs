//! Header rendering

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders},
    Frame,
};

use crate::theme::Theme;

pub fn draw_header(f: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .title(theme.copy.app_title)
        .borders(Borders::ALL)
        .style(theme.header);

    f.render_widget(block, area);
}
