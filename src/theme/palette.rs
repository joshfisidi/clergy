//! Raw color definitions — design tokens
//!
//! No semantics, no UI logic. Just colors.
//! Think of this as your design language.
//!
//! Color philosophy:
//! - Red is identity
//! - Cyan is information
//! - Amber is attention
//! - Green is success
//! - Gray is silence

use ratatui::style::Color;

/// Raw color palette — the design tokens
#[derive(Clone, Debug)]
pub struct Palette {
    // ─────────────────────────────────────────────────────────
    // Core identity
    // ─────────────────────────────────────────────────────────
    pub accent: Color,

    // ─────────────────────────────────────────────────────────
    // Semantic states
    // ─────────────────────────────────────────────────────────
    pub danger: Color,
    pub success: Color,
    pub warning: Color,

    // ─────────────────────────────────────────────────────────
    // Informational accents
    // ─────────────────────────────────────────────────────────
    pub info: Color,      // cyan / blue — metrics, data
    pub highlight: Color, // amber / orange — attention

    // ─────────────────────────────────────────────────────────
    // Neutral
    // ─────────────────────────────────────────────────────────
    pub dim: Color,
    pub fg: Color,
    pub bg: Color,
    pub panel_bg: Color,
}

impl Palette {
    /// CLERGY default palette — warm red identity, cool info tones
    pub fn clergy_default() -> Self {
        Self {
            // Identity — warm red-magenta (less harsh than pure red)
            accent: Color::Rgb(220, 50, 80),

            // Semantic states
            danger: Color::Rgb(255, 80, 80),
            success: Color::Rgb(80, 200, 120),
            warning: Color::Rgb(255, 180, 60), // amber

            // Informational
            info: Color::Rgb(80, 180, 255),      // cyan-blue
            highlight: Color::Rgb(255, 140, 60), // orange

            // Neutral
            dim: Color::DarkGray,
            fg: Color::White,
            bg: Color::Reset, // Respect terminal background
            panel_bg: Color::Rgb(14, 14, 18), // slightly cooler dark
        }
    }

    /// High contrast palette for accessibility
    #[allow(dead_code)]
    pub fn high_contrast() -> Self {
        Self {
            accent: Color::White,
            danger: Color::LightRed,
            success: Color::LightGreen,
            warning: Color::LightYellow,
            info: Color::LightCyan,
            highlight: Color::LightYellow,
            dim: Color::Gray,
            fg: Color::White,
            bg: Color::Black,
            panel_bg: Color::Black,
        }
    }

    /// Monochrome palette — no colors, just intensity
    #[allow(dead_code)]
    pub fn mono() -> Self {
        Self {
            accent: Color::White,
            danger: Color::White,
            success: Color::White,
            warning: Color::Gray,
            info: Color::Gray,
            highlight: Color::White,
            dim: Color::DarkGray,
            fg: Color::White,
            bg: Color::Reset,
            panel_bg: Color::Rgb(10, 10, 10),
        }
    }

    /// Mellow palette — calm blue/cyan/turquoise theme
    ///
    /// Designed for long sessions:
    /// - Low eye strain
    /// - Cool temperature
    /// - Clean metric readability
    #[allow(dead_code)]
    pub fn mellow() -> Self {
        Self {
            // Identity — soft turquoise
            accent: Color::Rgb(80, 200, 190),

            // Semantic states (muted, not alarming)
            danger: Color::Rgb(235, 110, 110),  // soft coral red
            success: Color::Rgb(90, 190, 160),  // sea green
            warning: Color::Rgb(220, 190, 120), // muted amber

            // Informational accents
            info: Color::Rgb(120, 200, 255),     // calm cyan-blue
            highlight: Color::Rgb(100, 180, 170), // teal highlight

            // Neutral tones
            dim: Color::Rgb(110, 120, 130),     // cool gray
            fg: Color::Rgb(220, 230, 240),      // soft white
            bg: Color::Reset,                   // respect terminal
            panel_bg: Color::Rgb(12, 16, 20),   // cool dark blue-gray
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::clergy_default()
    }
}
