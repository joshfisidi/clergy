//! Metrics panel rendering (8 compact boxes)
//!
//! Layout: 4 rows × 2 columns
//! [0] CPU        [1] Memory
//! [2] Disk       [3] Swap
//! [4] Uptime     [5] Load
//! [6] Processes  [7] Last Purge
//!
//! Features:
//! - Sparkline history (Unicode bars)
//! - Delta arrows (↑ ↓ →)
//! - Threshold coloring (green/yellow/red)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::system::{Level, Metrics, MetricsCollector, Sparkline, TrackedMetric};
use crate::theme::Theme;

// ─────────────────────────────────────────────────────────────
// Level-based styling (threshold coloring: green → amber → red)
// ─────────────────────────────────────────────────────────────

fn level_style(level: Level, theme: &Theme) -> Style {
    match level {
        Level::Normal => theme.metric_good,
        Level::Warning => theme.metric_warn,
        Level::Danger => theme.metric_bad,
    }
}

fn dot_style(level: Level, theme: &Theme) -> Style {
    match level {
        Level::Normal => theme.dot_normal,
        Level::Warning => theme.dot_warning,
        Level::Danger => theme.dot_danger,
    }
}

// ─────────────────────────────────────────────────────────────
// Dot grid — btop-style time × intensity matrix
// ─────────────────────────────────────────────────────────────
// X axis = time (older → newer, left to right)
// Y axis = intensity buckets (low → high, bottom to top)
// Each column fills from bottom up based on value
// No randomness, no animation — motion = history advancing

/// Dot glyphs (simple, single-cell)
const DOT_INACTIVE: &str = "·";
const DOT_ACTIVE: &str = "•";

/// Render dot grid (btop-style time × intensity matrix)
///
/// - `width`: number of history columns
/// - `height`: number of intensity rows
/// - Values fill bottom-up: 0-16% = 1 row, 16-33% = 2 rows, etc.
fn render_dot_grid(
    spark: &Sparkline,
    width: usize,
    height: usize,
    active_style: Style,
    inactive_style: Style,
) -> Vec<Line<'static>> {
    let samples: Vec<f64> = spark.values().collect();

    // Build a height × width grid of booleans (true = active)
    let mut grid = vec![vec![false; width]; height];

    // Fill columns from right (newest) to left (oldest)
    for (col_offset, &value) in samples.iter().rev().take(width).enumerate() {
        let col = width - 1 - col_offset;
        let normalized = (value / 100.0).clamp(0.0, 1.0);
        let filled_rows = (normalized * height as f64).round() as usize;

        // Fill from bottom up
        for row_offset in 0..filled_rows {
            let row = height - 1 - row_offset;
            grid[row][col] = true;
        }
    }

    // Convert grid to Lines (row 0 = top)
    grid.into_iter()
        .map(|row| {
            Line::from(
                row.into_iter()
                    .map(|active| {
                        if active {
                            Span::styled(DOT_ACTIVE, active_style)
                        } else {
                            Span::styled(DOT_INACTIVE, inactive_style)
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

/// Render dot grid with threshold-based coloring
fn render_usage_grid(
    spark: &Sparkline,
    level: Level,
    theme: &Theme,
    height: usize,
    _max_value: f64, // kept for API compatibility, grid assumes 0-100%
) -> Vec<Line<'static>> {
    let active_style = dot_style(level, theme);
    let inactive_style = theme.dim;

    // Width matches sparkline capacity (or available samples)
    let width = 18;

    render_dot_grid(spark, width, height, active_style, inactive_style)
}

/// Render dot grid for non-threshold metrics (info color)
fn render_usage_grid_info(
    spark: &Sparkline,
    theme: &Theme,
    height: usize,
    _max_value: f64,
) -> Vec<Line<'static>> {
    let active_style = theme.dot_info;
    let inactive_style = theme.dim;

    let width = 18;

    render_dot_grid(spark, width, height, active_style, inactive_style)
}

// ─────────────────────────────────────────────────────────────
// Tracked metric box (with sparkline, arrow, and threshold color)
// ─────────────────────────────────────────────────────────────

fn draw_tracked_metric(
    f: &mut Frame,
    area: Rect,
    title: &str,
    tracked: &TrackedMetric,
    detail: &str,
    theme: &Theme,
) {
    let block = Block::default()
        .title(Span::styled(title, theme.metric_label))
        .borders(Borders::ALL)
        .border_style(theme.metric_border);

    // Value with arrow and threshold color
    let value_style = level_style(tracked.level, theme);
    let value_text = format!("{:.0}% {}", tracked.pct, tracked.delta.arrow());

    // Usage grid (3 rows tall for compact display)
    let grid_rows = render_usage_grid(&tracked.spark, tracked.level, theme, 3, 100.0);

    let mut content = vec![Line::from(Span::styled(value_text, value_style))];
    content.extend(grid_rows);
    content.push(Line::from(Span::styled(detail.to_string(), theme.metric_label)));

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

// ─────────────────────────────────────────────────────────────
// Simple text metric box (no sparkline)
// ─────────────────────────────────────────────────────────────

fn draw_text_metric(f: &mut Frame, area: Rect, title: &str, value: &str, theme: &Theme) {
    let block = Block::default()
        .title(Span::styled(title, theme.metric_label))
        .borders(Borders::ALL)
        .border_style(theme.metric_border);

    let paragraph = Paragraph::new(Line::from(Span::styled(
        value.to_string(),
        theme.metric_value,
    )))
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

// ─────────────────────────────────────────────────────────────
// Text metric with sparkline (for load — non-threshold, uses info color)
// ─────────────────────────────────────────────────────────────

fn draw_text_metric_with_spark(
    f: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    spark: &Sparkline,
    theme: &Theme,
) {
    let block = Block::default()
        .title(Span::styled(title, theme.metric_label))
        .borders(Borders::ALL)
        .border_style(theme.metric_border);

    // Usage grid for load (normalized to max ~4.0 for typical systems)
    let grid_rows = render_usage_grid_info(spark, theme, 2, 4.0);

    let mut content = vec![Line::from(Span::styled(value.to_string(), theme.metric_value))];
    content.extend(grid_rows);

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

// ─────────────────────────────────────────────────────────────
// Main draw function
// ─────────────────────────────────────────────────────────────

pub fn draw_metrics(
    f: &mut Frame,
    area: Rect,
    metrics: &Metrics,
    collector: &MetricsCollector,
    theme: &Theme,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let mut regions = Vec::with_capacity(8);

    for row in rows.iter() {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(*row);

        regions.push(cols[0]);
        regions.push(cols[1]);
    }

    // ───── Row 1: CPU & Memory ─────
    draw_tracked_metric(
        f,
        regions[0],
        "CPU",
        &collector.cpu,
        &format!("{} cores", metrics.cpu.cores),
        theme,
    );

    draw_tracked_metric(
        f,
        regions[1],
        "Memory",
        &collector.mem,
        &format!(
            "{:.1}/{:.1}G",
            metrics.mem.used_mb as f64 / 1024.0,
            metrics.mem.total_mb as f64 / 1024.0
        ),
        theme,
    );

    // ───── Row 2: Disk & Swap ─────
    draw_tracked_metric(
        f,
        regions[2],
        "Disk",
        &collector.disk,
        &format!("{}G free", metrics.disk.free_gb),
        theme,
    );

    draw_tracked_metric(
        f,
        regions[3],
        "Swap",
        &collector.swap,
        &format!("{}M used", metrics.swap.used_mb),
        theme,
    );

    // ───── Row 3: Uptime & Load ─────
    draw_text_metric(f, regions[4], "Uptime", &metrics.uptime_human(), theme);

    draw_text_metric_with_spark(
        f,
        regions[5],
        "Load",
        &format!("{:.2}", metrics.load_1m),
        &collector.load.spark,
        theme,
    );

    // ───── Row 4: Processes & Last Purge ─────
    draw_text_metric(
        f,
        regions[6],
        "Procs",
        &metrics.process_count.to_string(),
        theme,
    );
    draw_text_metric(f, regions[7], "Purge", "—", theme);
}
