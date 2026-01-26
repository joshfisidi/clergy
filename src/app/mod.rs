//! Application state and logic
//!
//! This module contains the state machine, menu state, and animation state.
//! It does NOT contain rendering or input handling.

mod animation;
mod state;

pub use animation::LogoAnim;
pub use state::{ConfirmChoice, MenuState, SettingsState, UiState};
