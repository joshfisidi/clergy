use std::collections::VecDeque;
use sysinfo::{System, ProcessesToUpdate};

use super::cpu::{CpuMonitor, CpuStats};
use super::disk::{DiskMonitor, DiskStats};
use super::mem::{MemMonitor, MemStats};
use super::swap::{SwapMonitor, SwapStats};

// ─────────────────────────────────────────────────────────────
// Sparkline — rolling history with dot rendering
// ─────────────────────────────────────────────────────────────

const SPARKLINE_CAPACITY: usize = 20;

/// Braille-based blocks for dense terminal sparklines (btop-style)
const DOTS: &[&str] = &["⣀", "⣤", "⣶", "⣿"];

#[derive(Clone)]
pub struct Sparkline {
    values: VecDeque<f64>,
}

impl Sparkline {
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(SPARKLINE_CAPACITY),
        }
    }

    pub fn push(&mut self, v: f64) {
        if self.values.len() == SPARKLINE_CAPACITY {
            self.values.pop_front();
        }
        self.values.push_back(v.clamp(0.0, 100.0));
    }

    /// Get raw values for custom rendering
    pub fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.values.iter().cloned()
    }

    /// Convert value to braille block based on intensity
    pub fn dot_for_value(v: f64) -> &'static str {
        match v {
            v if v >= 85.0 => DOTS[3],
            v if v >= 70.0 => DOTS[2],
            v if v >= 40.0 => DOTS[1],
            _ => DOTS[0],
        }
    }

    /// Render sparkline string (optimized for ratatui)
    pub fn render_dots(&self) -> String {
        self.values.iter().map(|v| Self::dot_for_value(*v)).collect()
    }
}

impl Default for Sparkline {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────
// Delta — current vs previous with direction arrow
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default)]
pub struct Delta {
    pub current: f64,
    pub previous: f64,
}

impl Delta {
    pub fn update(&mut self, new_value: f64) {
        self.previous = self.current;
        self.current = new_value;
    }

    /// Direction arrow with epsilon tolerance
    pub fn arrow(&self) -> &'static str {
        const EPSILON: f64 = 0.5;
        match self.current - self.previous {
            d if d > EPSILON => "↑",
            d if d < -EPSILON => "↓",
            _ => "→",
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Threshold levels for conditional coloring
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Normal,
    Warning,
    Danger,
}

impl Level {
    /// Determine level from percentage (0-100)
    pub fn from_pct(pct: f64) -> Self {
        match pct {
            p if p >= 85.0 => Level::Danger,
            p if p >= 70.0 => Level::Warning,
            _ => Level::Normal,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Tracked metric — combines value, delta, sparkline, and level
// ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TrackedMetric {
    pub pct: f64,
    pub delta: Delta,
    pub spark: Sparkline,
    pub level: Level,
}

impl TrackedMetric {
    pub fn new() -> Self {
        Self {
            pct: 0.0,
            delta: Delta::default(),
            spark: Sparkline::new(),
            level: Level::Normal,
        }
    }

    pub fn update(&mut self, pct: f64) {
        self.pct = pct;
        self.delta.update(pct);
        self.spark.push(pct);
        self.level = Level::from_pct(pct);
    }
}

impl Default for TrackedMetric {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────
// Metrics snapshot — raw values for current frame
// ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Metrics {
    pub cpu: CpuStats,
    pub mem: MemStats,
    pub disk: DiskStats,
    pub swap: SwapStats,
    pub uptime_secs: u64,
    pub load_1m: f64,
    pub process_count: usize,
}

impl Metrics {
    /// Human-readable uptime (e.g., "2h 15m")
    pub fn uptime_human(&self) -> String {
        let secs = self.uptime_secs;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;

        if days > 0 {
            format!("{}d {}h", days, hours)
        } else if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        }
    }

    /// Compute CPU percentage
    pub fn cpu_pct(&self) -> f64 {
        self.cpu.usage as f64
    }

    /// Compute memory percentage
    pub fn mem_pct(&self) -> f64 {
        if self.mem.total_mb > 0 {
            (self.mem.used_mb as f64 / self.mem.total_mb as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Compute disk percentage
    pub fn disk_pct(&self) -> f64 {
        if self.disk.total_gb > 0 {
            let used = self.disk.total_gb.saturating_sub(self.disk.free_gb);
            (used as f64 / self.disk.total_gb as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Compute swap percentage
    pub fn swap_pct(&self) -> f64 {
        let total = self.swap.used_mb + self.swap.free_mb;
        if total > 0 {
            (self.swap.used_mb as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

// ─────────────────────────────────────────────────────────────
// MetricsCollector — gathers metrics + tracks history
// ─────────────────────────────────────────────────────────────

pub struct MetricsCollector {
    cpu_mon: CpuMonitor,
    mem_mon: MemMonitor,
    disk_mon: DiskMonitor,
    swap_mon: SwapMonitor,
    system: System,

    // Tracked metrics with sparklines and deltas
    pub cpu: TrackedMetric,
    pub mem: TrackedMetric,
    pub disk: TrackedMetric,
    pub swap: TrackedMetric,
    pub load: TrackedMetric,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            cpu_mon: CpuMonitor::new(),
            mem_mon: MemMonitor::new(),
            disk_mon: DiskMonitor::new(),
            swap_mon: SwapMonitor::new(),
            system: System::new(),
            cpu: TrackedMetric::new(),
            mem: TrackedMetric::new(),
            disk: TrackedMetric::new(),
            swap: TrackedMetric::new(),
            load: TrackedMetric::new(),
        }
    }

    pub fn refresh(&mut self) -> Metrics {
        self.system.refresh_processes(ProcessesToUpdate::All, false);

        let metrics = Metrics {
            cpu: self.cpu_mon.refresh(),
            mem: self.mem_mon.refresh(),
            disk: self.disk_mon.refresh(),
            swap: self.swap_mon.refresh(),
            uptime_secs: System::uptime(),
            load_1m: System::load_average().one,
            process_count: self.system.processes().len(),
        };

        // Update tracked metrics
        self.cpu.update(metrics.cpu_pct());
        self.mem.update(metrics.mem_pct());
        self.disk.update(metrics.disk_pct());
        self.swap.update(metrics.swap_pct());
        // Normalize load relative to CPU count (btop-style)
        let load_normalized =
            (metrics.load_1m / self.cpu_mon.cores() as f64) * 100.0;
        self.load.update(load_normalized.min(100.0));

        metrics
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
