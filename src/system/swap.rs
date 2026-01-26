use std::process::Command;

#[derive(Clone)]
pub struct SwapStats {
    pub used_mb: u64,
    pub free_mb: u64,
}

pub struct SwapMonitor;

impl SwapMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn refresh(&mut self) -> SwapStats {
        // Parse: vm.swapusage: total = 1024.00M  used = 512.00M  free = 512.00M
        let output = Command::new("sysctl")
            .arg("vm.swapusage")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let mut used_mb = 0u64;
        let mut free_mb = 0u64;

        for part in output.split_whitespace() {
            if part.ends_with('M') {
                if let Ok(val) = part.trim_end_matches('M').parse::<f64>() {
                    // The order is: total, used, free
                    if used_mb == 0 && free_mb == 0 {
                        // Skip total
                    } else if free_mb == 0 {
                        free_mb = val as u64;
                    }
                }
            }
        }

        // Better parsing approach
        let parts: Vec<&str> = output.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "used" && i + 2 < parts.len() {
                if let Some(val) = parts[i + 2].strip_suffix('M') {
                    used_mb = val.parse::<f64>().unwrap_or(0.0) as u64;
                }
            }
            if *part == "free" && i + 2 < parts.len() {
                if let Some(val) = parts[i + 2].strip_suffix('M') {
                    free_mb = val.parse::<f64>().unwrap_or(0.0) as u64;
                }
            }
        }

        SwapStats { used_mb, free_mb }
    }
}

impl Default for SwapMonitor {
    fn default() -> Self {
        Self::new()
    }
}
