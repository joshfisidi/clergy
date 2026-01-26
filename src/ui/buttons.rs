//! Button rendering
//!
//! Proper TUI buttons with three layers:
//! 1. Outer border (neutral or accent)
//! 2. Inner fill area (background color when active)
//! 3. Centered label (foreground color)
//!
//! Active button labels use theme.button_active_label for contrast.

use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::theme::Theme;

pub fn draw_button(f: &mut Frame, area: Rect, label: &str, active: bool, theme: &Theme) {
    let (border_style, fill_style, label_style) = if active {
        (
            theme.button_active_border,
            theme.button_active_fill,
            theme.button_active_label,
        )
    } else {
        (
            theme.button_inactive_border,
            theme.button_inactive_fill,
            theme.button_inactive_label,
        )
    };

    // 1️⃣ Draw the border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    f.render_widget(block.clone(), area);

    // 2️⃣ Compute inner area (the key to proper button rendering)
    let inner = block.inner(area);

    // 3️⃣ Fill the inner area explicitly (prevents bleed)
    if active && inner.width > 0 && inner.height > 0 {
        let fill_line = " ".repeat(inner.width as usize);
        for y in inner.y..inner.y + inner.height {
            let row = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            let fill = Paragraph::new(fill_line.clone()).style(fill_style);
            f.render_widget(fill, row);
        }
    }

    // 4️⃣ Draw the label centered on top
    let text = Paragraph::new(label)
        .style(label_style)
        .alignment(Alignment::Center);

    f.render_widget(text, inner);
}
