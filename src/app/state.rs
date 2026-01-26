//! Application state types
//!
//! These types represent the UI state machine — what screen we're on
//! and what choices are active.

use crate::model::PurgeData;

/// Main UI state machine
pub enum UiState {
    Dashboard,
    ConfirmPurge,
    Running,
    Result(PurgeData),
    Explain,
    Status,
    About,
    Settings(SettingsState),
    Error(String),
}

/// Settings screen navigation state
#[derive(Clone, Copy, Default)]
pub struct SettingsState {
    /// Currently selected setting row
    pub selected: usize,
}

impl SettingsState {
    /// Number of editable settings rows
    pub const ROW_COUNT: usize = 1; // Just theme for now

    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1).min(Self::ROW_COUNT - 1);
    }
}

/// Confirmation dialog choice
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmChoice {
    Confirm,
    Cancel,
}

impl ConfirmChoice {
    pub fn toggle(&mut self) {
        *self = match self {
            ConfirmChoice::Confirm => ConfirmChoice::Cancel,
            ConfirmChoice::Cancel => ConfirmChoice::Confirm,
        };
    }
}

/// Menu navigation state
pub struct MenuState {
    pub selected: usize,
    pub item_count: usize,
}

impl MenuState {
    pub fn new(item_count: usize) -> Self {
        Self {
            selected: 0,
            item_count,
        }
    }

    pub fn up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn down(&mut self) {
        self.selected = (self.selected + 1).min(self.item_count - 1);
    }
}
