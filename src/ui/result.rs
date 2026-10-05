//! A run receipt: command evidence first, observed system changes second.
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::branding::{clergy_logo_partial_styled, LOGO_HEIGHT};
use crate::model::{ActionResult, ActionStatus, PurgeData};
use crate::theme::Theme;

fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

fn amount(kib: u64) -> String {
    if kib >= 1024 * 1024 {
        format!("{:.3} GiB", kib as f64 / (1024.0 * 1024.0))
    } else if kib >= 1024 {
        format!("{:.2} MiB", kib as f64 / 1024.0)
    } else {
        format!("{} KiB", kib)
    }
}

fn change(before: u64, after: u64) -> String {
    let difference = after as i128 - before as i128;
    match difference.cmp(&0) {
        std::cmp::Ordering::Greater => format!("+{}", amount(difference as u64)),
        std::cmp::Ordering::Less => format!("−{}", amount((-difference) as u64)),
        std::cmp::Ordering::Equal => "No change".into(),
    }
}

fn section(lines: &mut Vec<Line<'static>>, title: &str, theme: &Theme) {
    lines.push(Line::default());
    lines.push(Line::styled(title.to_owned(), theme.highlight));
}

fn measurement(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    before: Option<u64>,
    after: Option<u64>,
    width: u16,
    theme: &Theme,
) {
    let delta = match (before, after) {
        (Some(before), Some(after)) => change(before, after),
        _ => "Unknown".into(),
    };
    let before_text = before.map(amount).unwrap_or_else(|| "Unavailable".into());
    let after_text = after.map(amount).unwrap_or_else(|| "Unavailable".into());
    if width >= 66 {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<16}", label), theme.metric_label),
            Span::styled(format!("{:>15}", before_text), theme.info),
            Span::styled(format!("{:>15}", after_text), theme.bold),
            Span::styled(
                format!("{:>18}", delta),
                if before == after {
                    theme.dim
                } else {
                    theme.info
                },
            ),
        ]));
    } else {
        lines.push(Line::styled(label.to_owned(), theme.metric_label));
        lines.push(Line::styled(
            format!("{} → {}  ({})", before_text, after_text, delta),
            theme.info,
        ));
    }
}

fn sample(mib: u64, available: Option<bool>) -> Option<u64> {
    (available != Some(false)).then(|| mib.saturating_mul(1024))
}

fn action_outcome(action: &ActionResult) -> String {
    let descriptions = match action.label.as_str() {
        "Directory cache" => Some((
            "Directory lookup cache cleared",
            "Directory lookup cache could not be cleared",
            "Directory lookup cache cleanup was not attempted",
        )),
        "DNS responder" => Some((
            "DNS responder refreshed",
            "DNS responder could not be refreshed",
            "DNS refresh was not attempted",
        )),
        "Time Machine snapshots" => Some((
            "Local snapshot cleanup completed",
            "Local snapshot cleanup could not be completed",
            "Local snapshot cleanup was not attempted",
        )),
        _ => None,
    };
    match (descriptions, action.status) {
        (Some((success, _, _)), ActionStatus::Succeeded) => success.into(),
        (Some((_, failure, _)), ActionStatus::Failed) => failure.into(),
        (Some((_, _, skipped)), ActionStatus::Skipped) => skipped.into(),
        (None, status) => format!(
            "{} {}",
            clean(&action.label),
            match status {
                ActionStatus::Succeeded => "completed",
                ActionStatus::Failed => "could not be completed",
                ActionStatus::Skipped => "was not attempted",
            }
        ),
    }
}

/// Pre-wrap styled lines using cell widths so scrolling has a stable, exact limit.
fn wrap(lines: Vec<Line<'static>>, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width.max(1));
    let mut result = Vec::new();
    for line in lines {
        let mut current = Vec::new();
        let mut used = 0;
        for span in line.spans {
            let mut part = String::new();
            for ch in span.content.chars() {
                let cells = ch.width().unwrap_or(0);
                if used + cells > width && used > 0 {
                    current.push(Span::styled(std::mem::take(&mut part), span.style));
                    result.push(Line::from(std::mem::take(&mut current)));
                    used = 0;
                }
                part.push(ch);
                used += cells;
            }
            current.push(Span::styled(part, span.style));
        }
        result.push(Line::from(current));
    }
    result
}

pub fn report_lines(data: &PurgeData, width: u16, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let completed = data.completed();
    let summary = if data.actions.is_empty() {
        "Saved legacy report · individual command details unavailable".to_owned()
    } else {
        let count = data
            .actions
            .iter()
            .filter(|a| a.status == ActionStatus::Succeeded)
            .count();
        format!(
            "{}  ·  {}/{} actions succeeded  ·  {}s elapsed",
            if completed {
                "PURGE COMPLETE"
            } else {
                "PURGE INCOMPLETE"
            },
            count,
            data.actions.len(),
            data.duration_seconds
        )
    };
    lines.push(Line::styled(
        summary,
        if completed {
            theme.success
        } else {
            theme.warning
        },
    ));
    lines.push(Line::styled(
        format!("{}  ·  {}", clean(&data.host), clean(&data.user)),
        theme.metric_label,
    ));
    lines.push(Line::styled(
        format!(
            "Started   {}",
            clean(data.started_at.as_deref().unwrap_or(&data.start_time))
        ),
        theme.info,
    ));
    if let Some(finished) = &data.finished_at {
        lines.push(Line::styled(
            format!("Finished  {}", clean(finished)),
            theme.info,
        ));
    } else {
        lines.push(Line::styled(
            format!("Duration  {}s", data.duration_seconds),
            theme.info,
        ));
    }
    if let Some(error) = &data.report_save_error {
        lines.push(Line::styled(
            format!("Could not save this report: {}", clean(error)),
            theme.warning,
        ));
    }

    section(&mut lines, "01 / ACTION JOURNAL", theme);
    if data.actions.is_empty() {
        lines.push(Line::styled(
            format!(
                "DNS flush: {} · Snapshot thinning: {}",
                if data.dns_flushed {
                    "reported successful"
                } else {
                    "not completed"
                },
                if data.snapshots_thinned {
                    "reported successful"
                } else {
                    "not completed"
                }
            ),
            theme.info,
        ));
        lines.push(Line::styled(
            "Individual cleanup details were not saved by this older version.",
            theme.dim,
        ));
    } else {
        for action in &data.actions {
            let (status, style) = match action.status {
                ActionStatus::Succeeded => ("✓ DONE", theme.success),
                ActionStatus::Failed => ("× FAILED", theme.danger),
                ActionStatus::Skipped => ("· SKIPPED", theme.dim),
            };
            let timing = match action.status {
                ActionStatus::Skipped => String::new(),
                _ => format!("  ·  {} ms", action.duration_ms),
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{}  ", status), style),
                Span::styled(action_outcome(action), theme.bold),
                Span::styled(timing, theme.dim),
            ]));
        }
        if data
            .actions
            .iter()
            .any(|action| action.status == ActionStatus::Skipped)
        {
            lines.push(Line::styled(
                "Remaining cleanup steps were skipped after an earlier failure.",
                theme.warning,
            ));
        }
        lines.push(Line::styled(
            "macOS does not report how many cached entries were cleared.",
            theme.dim,
        ));
    }

    section(&mut lines, "02 / MEASURED CHANGES", theme);
    if width >= 66 {
        lines.push(Line::styled(
            format!(
                "{:<16}{:>15}{:>15}{:>18}",
                "SAMPLE", "BEFORE", "AFTER", "CHANGE"
            ),
            theme.dim,
        ));
    }
    match (
        data.disk_available_before_kib,
        data.disk_available_after_kib,
    ) {
        (Some(before), Some(after)) => {
            measurement(
                &mut lines,
                "Disk free /",
                Some(before),
                Some(after),
                width,
                theme,
            );
            lines.push(Line::styled(
                format!("  Exact disk availability: {} → {} KiB", before, after),
                theme.dim,
            ));
        }
        _ => {
            let sample = |text: &str| {
                text.lines()
                    .nth(1)
                    .and_then(|line| line.split_whitespace().nth(3))
                    .unwrap_or("unavailable")
                    .to_owned()
            };
            lines.push(Line::styled(
                format!(
                    "Disk free /  {} → {}  (exact delta unavailable)",
                    sample(&data.disk_before),
                    sample(&data.disk_after)
                ),
                theme.dim,
            ));
        }
    }
    measurement(
        &mut lines,
        "RAM pages",
        sample(data.memory_before.used_mb, data.memory_before_available),
        sample(data.memory_after.used_mb, data.memory_after_available),
        width,
        theme,
    );
    measurement(
        &mut lines,
        "Free RAM",
        sample(data.memory_before.free_mb, data.memory_before_available),
        sample(data.memory_after.free_mb, data.memory_after_available),
        width,
        theme,
    );
    measurement(
        &mut lines,
        "Compressed RAM",
        sample(
            data.memory_before.compressed_mb,
            data.memory_before_available,
        ),
        sample(data.memory_after.compressed_mb, data.memory_after_available),
        width,
        theme,
    );
    measurement(
        &mut lines,
        "Swap used",
        sample(data.swap_before.used_mb, data.swap_before_available),
        sample(data.swap_after.used_mb, data.swap_after_available),
        width,
        theme,
    );
    measurement(
        &mut lines,
        "Swap free",
        sample(data.swap_before.free_mb, data.swap_before_available),
        sample(data.swap_after.free_mb, data.swap_after_available),
        width,
        theme,
    );

    section(&mut lines, "03 / TIME MACHINE SNAPSHOTS", theme);
    if let (Some(before), Some(after), Some(removed)) = (
        &data.snapshots_before,
        &data.snapshots_after,
        data.removed_snapshots(),
    ) {
        lines.push(Line::styled(
            format!(
                "{} before → {} after  ·  {} disappeared",
                before.len(),
                after.len(),
                removed.len()
            ),
            theme.info,
        ));
        if removed.is_empty() {
            lines.push(Line::styled(
                "No local Time Machine snapshots disappeared during this run.",
                theme.dim,
            ));
        } else {
            for name in removed {
                lines.push(Line::styled(format!("  − {}", clean(name)), theme.info));
            }
        }
    } else {
        lines.push(Line::styled(
            "Snapshot inventory unavailable; removed count is unknown.",
            theme.dim,
        ));
    }
    section(&mut lines, "READING THIS REPORT", theme);
    lines.push(Line::styled(
        "This cleanup refreshes lookup caches and trims local backups; it does not clear app RAM.",
        theme.dim,
    ));
    lines.push(Line::styled(
        "Disk/RAM deltas are observations during the run, not guaranteed reclamation.",
        theme.dim,
    ));
    lines.push(Line::styled(
        "RAM pages = active + inactive + speculative pages. Samples use binary units.",
        theme.dim,
    ));
    wrap(lines, width)
}

pub fn draw_result(
    f: &mut Frame,
    area: Rect,
    data: &PurgeData,
    scroll: &mut u16,
    logo_rows: usize,
    theme: &Theme,
) {
    let panel = super::panels::centered_panel(area);
    let logo_height = if panel.height >= 32 {
        LOGO_HEIGHT + 2
    } else {
        0
    };
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(logo_height), Constraint::Min(0)])
        .split(panel);
    if logo_height > 0 {
        f.render_widget(
            clergy_logo_partial_styled(logo_rows, theme.header),
            areas[0],
        );
    }
    let status = if data.completed() {
        "COMPLETE"
    } else {
        "INCOMPLETE"
    };
    let color = if data.completed() {
        theme.success
    } else {
        theme.warning
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.panel_border_focused)
        .title(Line::from(vec![
            Span::styled(format!(" {} ", theme.copy.result_title), theme.header),
            Span::styled(format!("· {} ", status), color),
        ]))
        .padding(Padding::new(1, 1, 1, 0));
    let inner = block.inner(areas[1]);
    let lines = report_lines(data, inner.width, theme);
    let max_scroll = lines
        .len()
        .saturating_sub(inner.height as usize)
        .min(u16::MAX as usize) as u16;
    *scroll = (*scroll).min(max_scroll);
    let block = if max_scroll > 0 {
        block.title_bottom(Line::styled(
            format!(
                " ↑↓ scroll · {}/{} ",
                u32::from(*scroll) + 1,
                u32::from(max_scroll) + 1
            ),
            theme.dim,
        ))
    } else {
        block
    };
    f.render_widget(
        Paragraph::new(lines).block(block).scroll((*scroll, 0)),
        areas[1],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, style::Modifier, Terminal};

    fn fixture() -> PurgeData {
        serde_json::from_str(include_str!("../../tests/fixtures/purge-report.json")).unwrap()
    }

    fn text(data: &PurgeData, width: u16) -> String {
        report_lines(data, width, &Theme::default())
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn precise_disk_change_survives_identical_rounded_df_values() {
        let data = fixture();
        assert_eq!(data.disk_before, data.disk_after);
        let output = text(&data, 88);
        assert!(output.contains("+2.00 MiB"));
        assert!(output.contains("−57.00 MiB"));
        assert!(output.contains("+54.00 MiB"));
        assert!(output.contains("1 disappeared"));
        assert!(output.contains("Local snapshot cleanup completed  ·  1513 ms"));
        assert!(!output.contains("space reclaimed"));
        assert!(!output.contains("sudo"));
        assert!(!output.contains("exit 0"));
        assert!(!output.contains("Thinned local snapshots:"));
        assert!(!output.contains("urgency"));
    }

    #[test]
    fn legacy_reports_remain_readable_without_invented_measurements() {
        let mut value = serde_json::to_value(fixture()).unwrap();
        for key in [
            "started_at",
            "finished_at",
            "actions",
            "disk_available_before_kib",
            "disk_available_after_kib",
            "snapshots_before",
            "snapshots_after",
        ] {
            value.as_object_mut().unwrap().remove(key);
        }
        let data: PurgeData = serde_json::from_value(value).unwrap();
        assert!(data.completed());
        let output = text(&data, 88);
        assert!(output.contains("Saved legacy report"));
        assert!(output.contains("exact delta unavailable"));
        assert!(output.contains("removed count is unknown"));
        assert!(!output.contains("+2.00 MiB"));
    }

    #[test]
    fn no_snapshots_and_failed_inventory_are_distinct() {
        let mut data = fixture();
        data.snapshots_before = Some(vec![]);
        data.snapshots_after = Some(vec![]);
        assert!(text(&data, 88).contains("0 before → 0 after"));
        data.snapshots_before = None;
        assert!(text(&data, 88).contains("removed count is unknown"));
        assert!(!text(&data, 88).contains("0 disappeared"));
    }

    #[test]
    fn unavailable_memory_and_swap_samples_never_look_like_observed_zero() {
        let mut data = fixture();
        data.memory_before_available = Some(false);
        data.memory_after_available = Some(false);
        data.swap_before_available = Some(false);
        data.swap_after_available = Some(false);
        let output = text(&data, 88);
        assert_eq!(output.matches("Unavailable").count(), 10);
        assert_eq!(output.matches("Unknown").count(), 5);
        assert!(!output.contains("No change"));
    }

    #[test]
    fn partial_failure_never_looks_complete() {
        let mut data = fixture();
        data.actions[1].status = ActionStatus::Failed;
        data.actions[1].exit_code = Some(1);
        data.actions[2].status = ActionStatus::Skipped;
        data.actions[2].exit_code = None;
        data.dns_flushed = false;
        data.snapshots_thinned = false;
        let output = text(&data, 88);
        assert!(!data.completed());
        assert!(output.contains("PURGE INCOMPLETE"));
        assert!(output.contains("× FAILED"));
        assert!(output.contains("· SKIPPED"));
        assert!(output.contains("DNS responder could not be refreshed"));
        assert!(output.contains("Local snapshot cleanup was not attempted"));
        assert!(!output.contains("sudo"));
        assert!(!output.contains("exit 1"));
    }

    #[test]
    fn report_fits_cell_width_and_scroll_reaches_final_notes() {
        for (width, height) in [(140, 50), (100, 36), (80, 24), (52, 20), (20, 10)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let data = fixture();
            let theme = Theme::default();
            let mut scroll = u16::MAX;
            terminal
                .draw(|f| draw_result(f, f.area(), &data, &mut scroll, 7, &theme))
                .unwrap();
            assert!(scroll < u16::MAX);
            assert!(report_lines(&data, width.saturating_sub(4), &theme)
                .iter()
                .all(|line| line.width() <= width.saturating_sub(4) as usize));
            if width >= 52 {
                let buffer = terminal.backend().buffer();
                let screen = buffer
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect::<String>();
                assert!(
                    screen.contains("binary units"),
                    "Final notes inaccessible at {width}x{height}"
                );
            }
        }
    }

    #[test]
    fn render_report_preview() {
        if std::env::var_os("CLERGY_RENDER_PREVIEW").is_none() {
            return;
        }
        let mut terminal = Terminal::new(TestBackend::new(100, 50)).unwrap();
        let data = fixture();
        let theme = Theme::default();
        let mut scroll = 0;
        terminal
            .draw(|f| draw_result(f, f.area(), &data, &mut scroll, 7, &theme))
            .unwrap();
        let cells: Vec<_> = terminal.backend().buffer().content().iter().map(|cell| {
            serde_json::json!({"symbol": cell.symbol(), "fg": format!("{:?}", cell.fg), "bold": cell.modifier.contains(Modifier::BOLD)})
        }).collect();
        std::fs::create_dir_all("target").unwrap();
        std::fs::write(
            "target/report-preview.json",
            serde_json::to_vec(&serde_json::json!({"width":100, "height":50, "cells":cells}))
                .unwrap(),
        )
        .unwrap();
    }
}
