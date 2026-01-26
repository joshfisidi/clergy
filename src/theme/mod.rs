//! Theme system
//!
//! Separates color definitions (Palette) from semantic styles (Theme).
//!
//! ```text
//! settings.toml → Palette → Theme → ui/*
//! ```
//!
//! UI code consumes Theme, never raw colors.
//! UI code consumes Copy, never hardcoded strings.

mod copy;
mod palette;
mod theme;

pub use copy::Copy;
pub use palette::Palette;
pub use theme::Theme;
