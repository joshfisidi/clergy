use crate::model::{MemoryStats, SwapStats};

/// Compute delta for compressed memory (negative = improvement)
pub fn compressed_delta(before: &MemoryStats, after: &MemoryStats) -> i64 {
    after.compressed_mb as i64 - before.compressed_mb as i64
}

/// Compute delta for used memory (negative = improvement)
pub fn memory_used_delta(before: &MemoryStats, after: &MemoryStats) -> i64 {
    after.used_mb as i64 - before.used_mb as i64
}

/// Compute delta for swap usage (negative = improvement)
pub fn swap_delta(before: &SwapStats, after: &SwapStats) -> i64 {
    after.used_mb as i64 - before.used_mb as i64
}

/// Parse disk available from df -h output (returns MB)
/// Example line: "/dev/disk1s1   466Gi  380Gi   82Gi    83%"
pub fn parse_disk_available(df_output: &str) -> Option<u64> {
    // Skip header line, get the data line
    let line = df_output.lines().nth(1)?;
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Available is typically the 4th column (index 3)
    if parts.len() < 4 {
        return None;
    }

    parse_size_to_mb(parts[3])
}

/// Parse size string (e.g., "82Gi", "3.4G", "512M") to MB
fn parse_size_to_mb(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Handle Gi/G suffix (gibibytes/gigabytes)
    if s.ends_with("Gi") {
        let num: f64 = s.trim_end_matches("Gi").parse().ok()?;
        return Some((num * 1024.0) as u64);
    }
    if s.ends_with('G') {
        let num: f64 = s.trim_end_matches('G').parse().ok()?;
        return Some((num * 1000.0) as u64);
    }

    // Handle Mi/M suffix (mebibytes/megabytes)
    if s.ends_with("Mi") {
        let num: f64 = s.trim_end_matches("Mi").parse().ok()?;
        return Some(num as u64);
    }
    if s.ends_with('M') {
        let num: f64 = s.trim_end_matches('M').parse().ok()?;
        return Some(num as u64);
    }

    // Handle Ki/K suffix (kibibytes/kilobytes)
    if s.ends_with("Ki") {
        let num: f64 = s.trim_end_matches("Ki").parse().ok()?;
        return Some((num / 1024.0) as u64);
    }
    if s.ends_with('K') {
        let num: f64 = s.trim_end_matches('K').parse().ok()?;
        return Some((num / 1000.0) as u64);
    }

    None
}

/// Compute disk space delta (positive = space reclaimed)
pub fn disk_delta_mb(before: &str, after: &str) -> i64 {
    let before_mb = parse_disk_available(before).unwrap_or(0);
    let after_mb = parse_disk_available(after).unwrap_or(0);

    // Positive means more space available = improvement
    after_mb as i64 - before_mb as i64
}

/// Format a delta value for display
/// Negative = improvement (↓), Positive = regression (↑)
pub fn format_delta(value: i64) -> String {
    if value < 0 {
        format!("↓ {}", value.abs())
    } else if value > 0 {
        format!("↑ {}", value)
    } else {
        "no change".into()
    }
}

/// Format disk delta (inverted: positive = good for disk)
pub fn format_disk_delta(value: i64) -> String {
    if value > 0 {
        format!("↑ {}", value)
    } else if value < 0 {
        format!("↓ {}", value.abs())
    } else {
        "no change".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size_to_mb("82Gi"), Some(82 * 1024));
        assert_eq!(parse_size_to_mb("3.4G"), Some(3400));
        assert_eq!(parse_size_to_mb("512M"), Some(512));
    }
}
