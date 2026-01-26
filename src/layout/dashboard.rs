//! Dashboard layout computation
//!
//! Pure Rect math — no rendering, no state, no widgets.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Pre-computed layout areas for the dashboard
///
/// ```text
/// Root (Vertical)
/// ├─ Header        (Length 3)
/// ├─ Body          (Min)
/// │  └─ Horizontal
/// │     ├─ Telemetry (Length 30)
/// │     └─ Main      (Min) — logo + menu + detail all live here
/// └─ Footer        (Length 3)
/// ```
pub struct DashboardLayout {
    pub header: Rect,
    pub telemetry: Rect,
    pub main: Rect,
    pub footer: Rect,
}

impl DashboardLayout {
    /// Compute layout from terminal area
    pub fn new(area: Rect) -> Self {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(10),   // body
                Constraint::Length(3), // footer
            ])
            .split(area);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // telemetry (left)
                Constraint::Min(0),     // main content (right)
            ])
            .split(root[1]);

        Self {
            header: root[0],
            telemetry: body[0],
            main: body[1],
            footer: root[2],
        }
    }
}
