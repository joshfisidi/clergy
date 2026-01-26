//! Semantic theme — what UI elements look like
//!
//! This is the bridge between raw colors (Palette) and UI code.
//! UI code consumes Theme, never Palette directly.
//! UI code consumes Copy, never hardcoded strings.

use ratatui::style::{Modifier, Style};

use super::copy::Copy;
use super::palette::Palette;
use crate::config::{Settings, ThemeKind};

/// Semantic theme — resolved styles for UI elements
#[derive(Clone)]
pub struct Theme {
    // ─────────────────────────────────────────────────────────
    // Layout chrome
    // ─────────────────────────────────────────────────────────
    pub header: Style,
    pub footer: Style,
    pub border: Style,
    pub border_focused: Style,
    pub panel_bg: Style,
    pub panel_border: Style,
    pub panel_border_focused: Style,

    // ─────────────────────────────────────────────────────────
    // Interactive elements (buttons use 3-layer model)
    // ─────────────────────────────────────────────────────────
    // Border styles
    pub button_active_border: Style,
    pub button_inactive_border: Style,
    // Fill styles (bg only, no fg)
    pub button_active_fill: Style,
    pub button_inactive_fill: Style,
    // Label styles (fg only, no bg)
    pub button_active_label: Style,
    pub button_inactive_label: Style,

    pub menu_selected: Style,
    pub menu_normal: Style,

    // ─────────────────────────────────────────────────────────
    // Metrics / data display
    // ─────────────────────────────────────────────────────────
    pub metric_value: Style,
    pub metric_label: Style,
    pub metric_bar_active: Style,
    pub metric_bar_inactive: Style,
    pub metric_border: Style,

    // Metric condition styles (threshold-based)
    pub metric_good: Style,
    pub metric_warn: Style,
    pub metric_bad: Style,

    // Dot sparkline styles (colored by level)
    pub dot_normal: Style,
    pub dot_warning: Style,
    pub dot_danger: Style,
    pub dot_info: Style, // for non-threshold metrics

    // ─────────────────────────────────────────────────────────
    // Semantic states
    // ─────────────────────────────────────────────────────────
    pub danger: Style,
    pub success: Style,
    pub warning: Style,
    pub dim: Style,
    pub bold: Style,

    // ─────────────────────────────────────────────────────────
    // Informational accents
    // ─────────────────────────────────────────────────────────
    pub info: Style,      // cyan / blue — data, metrics
    pub highlight: Style, // amber / orange — attention

    // ─────────────────────────────────────────────────────────
    // Theme-aware copy (text variants)
    // ─────────────────────────────────────────────────────────
    pub copy: Copy,
}

impl Theme {
    /// Create a theme from user settings
    ///
    /// This is the primary constructor — resolves ThemeKind to Palette,
    /// then applies any color overrides from settings.
    pub fn from_settings(settings: &Settings) -> Self {
        let base_palette = match settings.theme {
            ThemeKind::Clergy => Palette::clergy_default(),
            ThemeKind::Mellow => Palette::mellow(),
            ThemeKind::Mono => Palette::mono(),
            ThemeKind::HighContrast => Palette::high_contrast(),
        };

        // Apply color overrides if present
        let palette = Palette {
            accent: settings.accent_override().unwrap_or(base_palette.accent),
            danger: settings.danger_override().unwrap_or(base_palette.danger),
            success: settings.success_override().unwrap_or(base_palette.success),
            warning: settings.warning_override().unwrap_or(base_palette.warning),
            info: base_palette.info,
            highlight: base_palette.highlight,
            dim: settings.dim_override().unwrap_or(base_palette.dim),
            fg: base_palette.fg,
            bg: base_palette.bg,
            panel_bg: base_palette.panel_bg,
        };

        // Resolve copy based on theme kind
        let copy = match settings.theme {
            ThemeKind::Clergy => Copy::clergy(),
            ThemeKind::Mellow => Copy::mellow(),
            ThemeKind::Mono => Copy::minimal(),
            ThemeKind::HighContrast => Copy::high_contrast(),
        };

        Self::from_palette_with_copy(&palette, copy)
    }

    /// Create a theme from a palette (low-level, uses default copy)
    pub fn from_palette(p: &Palette) -> Self {
        Self::from_palette_with_copy(p, Copy::default())
    }

    /// Create a theme from a palette with specific copy
    fn from_palette_with_copy(p: &Palette, copy: Copy) -> Self {
        Self {
            // Layout chrome
            header: Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),

            footer: Style::default().fg(p.dim),

            border: Style::default().fg(p.dim),

            border_focused: Style::default().fg(p.accent),

            panel_bg: Style::default().bg(p.panel_bg),

            panel_border: Style::default().fg(p.dim),

            panel_border_focused: Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD),

            // Buttons (3-layer model: border, fill, label)
            button_active_border: Style::default().fg(p.accent),
            button_inactive_border: Style::default().fg(p.dim),

            // Fill — bg only, never define fg
            button_active_fill: Style::default().bg(p.accent),
            button_inactive_fill: Style::default(),

            // Label — fg only, contrasts with accent fill
            button_active_label: Style::default()
                .fg(p.panel_bg)
                .add_modifier(Modifier::BOLD),
            button_inactive_label: Style::default().fg(p.dim),

            // Menu — accent color throughout, selected has emphasis
            menu_selected: Style::default()
                .fg(p.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),

            menu_normal: Style::default().fg(p.accent),

            // Metrics — accent for borders (identity), info for data
            metric_value: Style::default()
                .fg(p.info)
                .add_modifier(Modifier::BOLD),

            metric_label: Style::default().fg(p.dim),

            metric_bar_active: Style::default().fg(p.info),

            metric_bar_inactive: Style::default().fg(p.dim),

            metric_border: Style::default().fg(p.accent),

            // Metric condition styles (threshold-based)
            metric_good: Style::default().fg(p.success),
            metric_warn: Style::default().fg(p.warning),
            metric_bad: Style::default()
                .fg(p.danger)
                .add_modifier(Modifier::BOLD),

            // Dot grid styles — normal uses accent (theme identity), warning/danger semantic
            dot_normal: Style::default().fg(p.accent),
            dot_warning: Style::default()
                .fg(p.warning)
                .add_modifier(Modifier::BOLD),
            dot_danger: Style::default()
                .fg(p.danger)
                .add_modifier(Modifier::BOLD),
            dot_info: Style::default().fg(p.info),

            // Semantic states
            danger: Style::default()
                .fg(p.danger)
                .add_modifier(Modifier::BOLD),

            success: Style::default().fg(p.success),

            warning: Style::default().fg(p.warning),

            dim: Style::default().fg(p.dim),

            bold: Style::default().add_modifier(Modifier::BOLD),

            // Informational accents
            info: Style::default().fg(p.info),

            highlight: Style::default()
                .fg(p.highlight)
                .add_modifier(Modifier::BOLD),

            // Copy
            copy,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_palette(&Palette::default())
    }
}
