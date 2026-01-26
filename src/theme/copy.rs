//! Theme-aware copy/text — semantic tone control
//!
//! UI code should never own words. This struct provides
//! theme-specific text variants for all user-visible strings.

/// Theme-aware text variants
#[derive(Clone)]
pub struct Copy {
    // ─────────────────────────────────────────────────────────
    // Titles
    // ─────────────────────────────────────────────────────────
    pub app_title: &'static str,
    pub menu_title: &'static str,
    pub confirm_title: &'static str,
    pub settings_title: &'static str,
    pub about_title: &'static str,
    pub status_title: &'static str,
    pub result_title: &'static str,
    pub error_title: &'static str,
    pub explain_title: &'static str,

    // ─────────────────────────────────────────────────────────
    // Menu prefixes
    // ─────────────────────────────────────────────────────────
    pub menu_prefix_selected: &'static str,
    pub menu_prefix_normal: &'static str,

    // ─────────────────────────────────────────────────────────
    // Footer hints
    // ─────────────────────────────────────────────────────────
    pub footer_nav: &'static str,
    pub footer_select: &'static str,
    pub footer_back: &'static str,
    pub footer_quit: &'static str,

    // ─────────────────────────────────────────────────────────
    // Status messages
    // ─────────────────────────────────────────────────────────
    pub running_message: &'static str,
}

impl Copy {
    /// Default copy for the Clergy theme
    pub fn clergy() -> Self {
        Self {
            app_title: "CLERGY · macOS System Health",
            menu_title: "Menu",
            confirm_title: "Confirm Purge",
            settings_title: "Settings",
            about_title: "About",
            status_title: "System Status",
            result_title: "Result",
            error_title: "Error",
            explain_title: "What CLERGY Does",

            menu_prefix_selected: "▶ ",
            menu_prefix_normal: "  ",

            footer_nav: "↑↓ navigate",
            footer_select: "Enter select",
            footer_back: "Esc back",
            footer_quit: "q quit",

            running_message: "Running system purge…",
        }
    }

    /// Minimal copy — less visual noise
    #[allow(dead_code)]
    pub fn minimal() -> Self {
        Self {
            app_title: "CLERGY",
            menu_title: "Menu",
            confirm_title: "Confirm",
            settings_title: "Settings",
            about_title: "About",
            status_title: "Status",
            result_title: "Result",
            error_title: "Error",
            explain_title: "Info",

            menu_prefix_selected: "> ",
            menu_prefix_normal: "  ",

            footer_nav: "↑↓",
            footer_select: "↵",
            footer_back: "Esc",
            footer_quit: "q",

            running_message: "Running…",
        }
    }

    /// High contrast copy — clearer, more explicit
    #[allow(dead_code)]
    pub fn high_contrast() -> Self {
        Self {
            app_title: "CLERGY - macOS System Health",
            menu_title: "MENU",
            confirm_title: "CONFIRM PURGE",
            settings_title: "SETTINGS",
            about_title: "ABOUT",
            status_title: "SYSTEM STATUS",
            result_title: "RESULT",
            error_title: "ERROR",
            explain_title: "WHAT CLERGY DOES",

            menu_prefix_selected: "[>] ",
            menu_prefix_normal: "[ ] ",

            footer_nav: "Up/Down to navigate",
            footer_select: "Enter to select",
            footer_back: "Escape to go back",
            footer_quit: "Q to quit",

            running_message: "Running system purge, please wait…",
        }
    }

    /// Mellow copy — calm, understated tone
    #[allow(dead_code)]
    pub fn mellow() -> Self {
        Self {
            app_title: "clergy · system health",
            menu_title: "menu",
            confirm_title: "confirm",
            settings_title: "settings",
            about_title: "about",
            status_title: "status",
            result_title: "result",
            error_title: "error",
            explain_title: "info",

            menu_prefix_selected: "› ",
            menu_prefix_normal: "  ",

            footer_nav: "↑↓ nav",
            footer_select: "↵ select",
            footer_back: "esc back",
            footer_quit: "q quit",

            running_message: "running purge…",
        }
    }
}

impl Default for Copy {
    fn default() -> Self {
        Self::clergy()
    }
}
