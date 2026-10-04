//! Input handling
//!
//! All key handling logic extracted from the main loop.

use crossterm::event::KeyCode;

use crate::app::{ConfirmChoice, MenuState, SettingsState, UiState};
use crate::config::Settings;

/// Result of handling a key event
pub enum InputResult {
    /// Continue the main loop
    Continue,
    /// Theme was changed, regenerate it
    ThemeChanged,
    /// The user confirmed a purge; the runtime must authenticate before starting it.
    StartPurge,
    /// Exit the application
    Quit,
}

/// Handle key input for the confirmation dialog
pub fn handle_confirm_key(
    key: KeyCode,
    state: &mut UiState,
    confirm_choice: &mut ConfirmChoice,
    settings: &mut Settings,
) -> InputResult {
    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            confirm_choice.toggle();
        }
        KeyCode::Enter => match confirm_choice {
            ConfirmChoice::Confirm => match crate::actions::can_run_purge(settings) {
                Ok(()) => {
                    return InputResult::StartPurge;
                }
                Err(remaining) => {
                    *state = UiState::Error(format!(
                        "Purge recently performed.\nPlease wait {} seconds before running again.",
                        remaining.as_secs()
                    ));
                }
            },
            ConfirmChoice::Cancel => {
                *state = UiState::Dashboard;
            }
        },
        KeyCode::Esc => {
            *state = UiState::Dashboard;
        }
        _ => {}
    }
    InputResult::Continue
}

/// Handle key input for the main dashboard/menu
pub fn handle_menu_key(
    key: KeyCode,
    state: &mut UiState,
    menu: &mut MenuState,
    confirm_choice: &mut ConfirmChoice,
) -> InputResult {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            menu.up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            menu.down();
        }
        KeyCode::Enter => match menu.selected {
            0 => {
                *state = UiState::ConfirmPurge;
                *confirm_choice = ConfirmChoice::Confirm;
            }
            1 => {
                if let Some(data) = crate::actions::load_last() {
                    *state = UiState::Result(data);
                } else {
                    *state = UiState::Error("No previous purge found.".into());
                }
            }
            2 => *state = UiState::Explain,
            3 => *state = UiState::Status,
            4 => *state = UiState::About,
            5 => *state = UiState::Settings(SettingsState::new()),
            6 => return InputResult::Quit,
            _ => {}
        },
        KeyCode::Esc | KeyCode::Backspace => {
            *state = UiState::Dashboard;
        }
        KeyCode::Char('q') => return InputResult::Quit,
        _ => {}
    }
    InputResult::Continue
}

/// Handle key input based on current state
pub fn handle_key(
    key: KeyCode,
    state: &mut UiState,
    menu: &mut MenuState,
    confirm_choice: &mut ConfirmChoice,
    settings: &mut Settings,
) -> InputResult {
    // A second purge or navigation must not orphan an operation still in progress.
    if matches!(state, UiState::Running) {
        return InputResult::Continue;
    }

    // Handle confirmation screen inputs first
    if matches!(state, UiState::ConfirmPurge) {
        return handle_confirm_key(key, state, confirm_choice, settings);
    }

    // Handle settings screen
    if let UiState::Settings(ref mut settings_state) = state {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                settings_state.up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                settings_state.down();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // Cycle theme backward
                if settings_state.selected == 0 {
                    settings.theme = settings.theme.prev();
                    return InputResult::ThemeChanged;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                // Cycle theme forward
                if settings_state.selected == 0 {
                    settings.theme = settings.theme.next();
                    return InputResult::ThemeChanged;
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                *state = UiState::Dashboard;
            }
            KeyCode::Char('q') => return InputResult::Quit,
            _ => {}
        }
        return InputResult::Continue;
    }

    // Normal menu handling
    handle_menu_key(key, state, menu, confirm_choice)
}
