//! UI rendering
//!
//! Pure drawing functions — no state, no input handling.
//! All draw functions take Frame, Rect, and Theme; they don't decide colors.

mod buttons;
mod footer;
mod header;
mod metrics;
mod panels;

pub use footer::draw_footer;
pub use header::draw_header;
pub use metrics::draw_metrics;
pub use panels::{
    draw_about, draw_confirm_purge, draw_error, draw_explain, draw_menu_column,
    draw_result, draw_running, draw_settings, draw_status, MENU_ITEMS,
};
