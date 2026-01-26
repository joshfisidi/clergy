//! Branding — identity and visual assets
//!
//! This module contains:
//! - About text and version info
//! - Logo rendering

mod about;
mod logo;

pub use about::{about_text, attribution, VERSION};
pub use logo::{clergy_logo_partial_styled, LOGO_HEIGHT};
