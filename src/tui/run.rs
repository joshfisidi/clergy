//! TUI orchestration
//!
//! This is the main loop that ties together:
//! - app (state)
//! - layout (Rect math)
//! - ui (rendering)
//! - input (key handling)
//! - theme (visual system)

use std::time::Duration;

use crossbeam_channel::{Receiver, TryRecvError};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use super::terminal::TerminalSession;

use crate::app::{ConfirmChoice, LogoAnim, MenuState, UiState};
use crate::config::Settings;
use crate::input::{handle_key, InputResult};
use crate::layout::DashboardLayout;
use crate::model::PurgeData;
use crate::system::MetricsCollector;
use crate::theme::Theme;
use crate::ui::{
    draw_about, draw_confirm_purge, draw_error, draw_explain, draw_footer, draw_header,
    draw_menu_column, draw_metrics, draw_result, draw_running, draw_settings, draw_status,
    MENU_ITEMS,
};

// ─────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────

const FRAME_MS: u64 = 120;

// ─────────────────────────────────────────────────────────────
// PUBLIC ENTRY POINTS
// ─────────────────────────────────────────────────────────────

pub fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    // Load settings (mutable for live theme changes)
    let mut settings = crate::config::load();

    // Create theme from settings (regenerated when settings change)
    let mut theme = Theme::from_settings(&settings);

    let mut session = TerminalSession::new()?;

    let mut state = UiState::Dashboard;
    let mut menu = MenuState::new(MENU_ITEMS.len());
    let mut confirm_choice = ConfirmChoice::Confirm;
    let mut collector = MetricsCollector::new();
    let mut logo_anim = LogoAnim::new();

    ui_loop(
        &mut session,
        &mut state,
        &mut menu,
        &mut confirm_choice,
        &mut collector,
        &mut logo_anim,
        &mut settings,
        &mut theme,
    )
}

/// Display purge results in TUI (called from `clergy purge`)
pub fn render_purge_ui(data: &PurgeData) -> Result<(), Box<dyn std::error::Error>> {
    // Load settings and create theme
    let settings = crate::config::load();
    let theme = Theme::from_settings(&settings);

    let mut session = TerminalSession::new()?;
    let mut collector = MetricsCollector::new();
    let mut logo_anim = LogoAnim::new();

    loop {
        if session.interrupted() {
            return Ok(());
        }
        let metrics = collector.refresh();
        logo_anim.tick();

        session.terminal.draw(|f| {
            let layout = DashboardLayout::new(f.area());
            draw_header(f, layout.header, &theme);
            draw_metrics(f, layout.telemetry, &metrics, &collector, &theme);
            draw_result(f, layout.main, data, logo_anim.visible_rows, &theme);
            draw_footer(f, layout.footer, &theme, false);
        })?;

        if event::poll(Duration::from_millis(FRAME_MS))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    break;
                }
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// MAIN LOOP
// ─────────────────────────────────────────────────────────────

fn ui_loop(
    session: &mut TerminalSession,
    state: &mut UiState,
    menu: &mut MenuState,
    confirm_choice: &mut ConfirmChoice,
    collector: &mut MetricsCollector,
    logo_anim: &mut LogoAnim,
    settings: &mut Settings,
    theme: &mut Theme,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut purge_worker: Option<Receiver<Result<PurgeData, String>>> = None;
    loop {
        // An external interrupt must not abandon a purge that is still running.
        if session.interrupted() && purge_worker.is_none() {
            return Ok(());
        }
        if let Some(receiver) = &purge_worker {
            match receiver.try_recv() {
                Ok(Ok(data)) => {
                    crate::actions::mark_purge_run();
                    let _ = crate::actions::save_last(&data);
                    *state = UiState::Result(data);
                    purge_worker = None;
                }
                Ok(Err(error)) => {
                    *state = UiState::Error(error);
                    purge_worker = None;
                }
                Err(TryRecvError::Disconnected) => {
                    *state = UiState::Error("Purge worker stopped unexpectedly.".into());
                    purge_worker = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }

        let metrics = collector.refresh();
        logo_anim.tick();

        session.terminal.draw(|f| {
            let layout = DashboardLayout::new(f.area());

            draw_header(f, layout.header, theme);
            draw_metrics(f, layout.telemetry, &metrics, collector, theme);
            draw_footer(f, layout.footer, theme, matches!(state, UiState::Running));

            let logo_rows = logo_anim.visible_rows;

            match state {
                UiState::Dashboard => {
                    draw_menu_column(f, layout.main, menu.selected, logo_rows, theme)
                }
                UiState::ConfirmPurge => {
                    draw_confirm_purge(f, layout.main, confirm_choice, logo_rows, theme)
                }
                UiState::Running => draw_running(f, layout.main, logo_rows, theme),
                UiState::Result(data) => draw_result(f, layout.main, data, logo_rows, theme),
                UiState::Explain => draw_explain(f, layout.main, logo_rows, theme),
                UiState::Status => draw_status(f, layout.main, &metrics, logo_rows, theme),
                UiState::About => draw_about(f, layout.main, logo_rows, theme),
                UiState::Settings(settings_state) => {
                    draw_settings(f, layout.main, settings, *settings_state, logo_rows, theme)
                }
                UiState::Error(msg) => draw_error(f, layout.main, msg, logo_rows, theme),
            }
        })?;

        if event::poll(Duration::from_millis(FRAME_MS))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match handle_key(key.code, state, menu, confirm_choice, settings) {
                    InputResult::Quit => return Ok(()),
                    InputResult::StartPurge => match session.authenticate()? {
                        Ok(()) => {
                            *state = UiState::Running;
                            purge_worker = Some(crate::worker::spawn_purge_worker());
                        }
                        Err(error) => *state = UiState::Error(error.to_string()),
                    },
                    InputResult::ThemeChanged => {
                        // Regenerate theme from updated settings
                        *theme = Theme::from_settings(settings);
                    }
                    InputResult::Continue => {}
                }
            }
        }
    }
}
