use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────
// COLOR ABSTRACTION (terminal-safe)
// ─────────────────────────────────────────────────────────────

/// Serializable color names that map to terminal-safe colors.
/// No RGB yet — future-proofing without scope creep.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorName {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
}

impl ColorName {
    pub fn to_color(&self) -> Color {
        match self {
            ColorName::Black => Color::Black,
            ColorName::Red => Color::Red,
            ColorName::Green => Color::Green,
            ColorName::Yellow => Color::Yellow,
            ColorName::Blue => Color::Blue,
            ColorName::Magenta => Color::Magenta,
            ColorName::Cyan => Color::Cyan,
            ColorName::White => Color::White,
            ColorName::Gray => Color::DarkGray,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// THEME KIND (preset selection)
// ─────────────────────────────────────────────────────────────

/// Theme presets — user picks one, system resolves styles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeKind {
    #[default]
    Clergy,
    Mellow,
    Mono,
    HighContrast,
}

impl ThemeKind {
    /// All available theme kinds for UI iteration
    pub const ALL: &'static [ThemeKind] = &[
        ThemeKind::Clergy,
        ThemeKind::Mellow,
        ThemeKind::Mono,
        ThemeKind::HighContrast,
    ];

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            ThemeKind::Clergy => "Clergy",
            ThemeKind::Mellow => "Mellow",
            ThemeKind::Mono => "Mono",
            ThemeKind::HighContrast => "High Contrast",
        }
    }

    /// Cycle to next theme
    pub fn next(&self) -> Self {
        match self {
            ThemeKind::Clergy => ThemeKind::Mellow,
            ThemeKind::Mellow => ThemeKind::Mono,
            ThemeKind::Mono => ThemeKind::HighContrast,
            ThemeKind::HighContrast => ThemeKind::Clergy,
        }
    }

    /// Cycle to previous theme
    pub fn prev(&self) -> Self {
        match self {
            ThemeKind::Clergy => ThemeKind::HighContrast,
            ThemeKind::Mellow => ThemeKind::Clergy,
            ThemeKind::Mono => ThemeKind::Mellow,
            ThemeKind::HighContrast => ThemeKind::Mono,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// THEME OVERRIDES (optional customization)
// ─────────────────────────────────────────────────────────────

/// User-facing color overrides.
/// All fields are optional — preset colors are used when None.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeOverrides {
    pub accent: Option<ColorName>,
    pub success: Option<ColorName>,
    pub warning: Option<ColorName>,
    pub danger: Option<ColorName>,
    pub dim: Option<ColorName>,
}

// ─────────────────────────────────────────────────────────────
// BEHAVIORAL SETTINGS
// ─────────────────────────────────────────────────────────────

/// Purge-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeSettings {
    /// Cooldown between purge runs in seconds
    #[serde(default = "default_cooldown_secs")]
    pub cooldown_secs: u64,
    /// Whether to show confirmation dialog before purge
    #[serde(default = "default_confirm")]
    pub confirm: bool,
}

fn default_cooldown_secs() -> u64 {
    60
}

fn default_confirm() -> bool {
    true
}

impl Default for PurgeSettings {
    fn default() -> Self {
        Self {
            cooldown_secs: default_cooldown_secs(),
            confirm: default_confirm(),
        }
    }
}

// ─────────────────────────────────────────────────────────────
// SETTINGS (root)
// ─────────────────────────────────────────────────────────────

/// Root settings struct.
/// Additive only — never remove fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Theme preset (clergy, mono, high_contrast)
    #[serde(default)]
    pub theme: ThemeKind,
    /// Optional color overrides (applied on top of preset)
    #[serde(default)]
    pub colors: ThemeOverrides,
    /// Purge behavior
    #[serde(default)]
    pub purge: PurgeSettings,
}

impl Settings {
    // ─────────────────────────────────────────────────────────
    // Theme accessors (color overrides)
    // ─────────────────────────────────────────────────────────

    /// Accent color override.
    pub fn accent_override(&self) -> Option<Color> {
        self.colors.accent.as_ref().map(|c| c.to_color())
    }

    /// Success color override.
    pub fn success_override(&self) -> Option<Color> {
        self.colors.success.as_ref().map(|c| c.to_color())
    }

    /// Warning color override.
    pub fn warning_override(&self) -> Option<Color> {
        self.colors.warning.as_ref().map(|c| c.to_color())
    }

    /// Danger color override.
    pub fn danger_override(&self) -> Option<Color> {
        self.colors.danger.as_ref().map(|c| c.to_color())
    }

    /// Dim color override.
    pub fn dim_override(&self) -> Option<Color> {
        self.colors.dim.as_ref().map(|c| c.to_color())
    }

    // ─────────────────────────────────────────────────────────
    // Behavioral accessors
    // ─────────────────────────────────────────────────────────

    /// Purge cooldown as Duration
    pub fn purge_cooldown(&self) -> Duration {
        Duration::from_secs(self.purge.cooldown_secs)
    }

    /// Whether to confirm before purge
    pub fn confirm_purge(&self) -> bool {
        self.purge.confirm
    }
}

// ─────────────────────────────────────────────────────────────
// PERSISTENCE
// ─────────────────────────────────────────────────────────────

/// Resolve settings path: ~/.config/clergy/config.toml
pub fn settings_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("clergy");
    dir.push("config.toml");
    dir
}

/// Ensure config directory exists
fn ensure_config_dir() -> std::io::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Load settings from disk, falling back to defaults.
/// If the file doesn't exist or is malformed, returns default settings.
pub fn load() -> Settings {
    let path = settings_path();

    if !path.exists() {
        return Settings::default();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

/// Persist settings to disk.
pub fn save(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    ensure_config_dir()?;
    let path = settings_path();
    let toml = toml::to_string_pretty(settings)?;
    fs::write(path, toml)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_uses_clergy_theme() {
        let s = Settings::default();
        assert_eq!(s.theme, ThemeKind::Clergy);
    }

    #[test]
    fn default_settings_has_no_color_overrides() {
        let s = Settings::default();
        assert!(s.accent_override().is_none());
        assert!(s.success_override().is_none());
        assert!(s.warning_override().is_none());
        assert!(s.danger_override().is_none());
        assert!(s.dim_override().is_none());
    }

    #[test]
    fn default_purge_settings() {
        let s = Settings::default();
        assert_eq!(s.purge.cooldown_secs, 60);
        assert!(s.purge.confirm);
        assert_eq!(s.purge_cooldown(), Duration::from_secs(60));
    }

    #[test]
    fn color_name_to_color_mapping() {
        assert_eq!(ColorName::Cyan.to_color(), Color::Cyan);
        assert_eq!(ColorName::Gray.to_color(), Color::DarkGray);
    }

    #[test]
    fn deserialize_toml_with_theme_kind() {
        let toml = r#"
theme = "mono"

[colors]
accent = "cyan"
dim = "gray"

[purge]
cooldown_secs = 120
confirm = false
"#;
        let s: Settings = toml::from_str(toml).unwrap();
        assert_eq!(s.theme, ThemeKind::Mono);
        assert_eq!(s.accent_override(), Some(Color::Cyan));
        assert_eq!(s.dim_override(), Some(Color::DarkGray));
        assert_eq!(s.purge.cooldown_secs, 120);
        assert!(!s.purge.confirm);
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let toml = r#"
[purge]
cooldown_secs = 30
"#;
        let s: Settings = toml::from_str(toml).unwrap();
        // Theme should use default (Clergy)
        assert_eq!(s.theme, ThemeKind::Clergy);
        // Colors should have no overrides
        assert!(s.accent_override().is_none());
        // Purge confirm should use default (true)
        assert!(s.purge.confirm);
        // cooldown_secs should be overridden
        assert_eq!(s.purge.cooldown_secs, 30);
    }

    #[test]
    fn theme_kind_names() {
        assert_eq!(ThemeKind::Clergy.name(), "Clergy");
        assert_eq!(ThemeKind::Mono.name(), "Mono");
        assert_eq!(ThemeKind::HighContrast.name(), "High Contrast");
    }
}
