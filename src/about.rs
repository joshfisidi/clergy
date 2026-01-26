/// About section for the TUI application.
/// Displays authorship and ownership only.

pub const ABOUT_LINES: &[&str] = &[
    "Fisidi Labs",
    "A division of Fisidi Inc.",
];

pub fn about_lines() -> &'static [&'static str] {
    ABOUT_LINES
}
