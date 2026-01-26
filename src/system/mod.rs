//! System telemetry — OS metrics and monitoring
//!
//! This module contains all system inspection code:
//! CPU, memory, disk, swap, and derived metrics.

mod cpu;
mod delta;
mod disk;
mod mem;
mod metrics;
mod swap;

pub use cpu::{CpuMonitor, CpuStats};
pub use disk::{DiskMonitor, DiskStats};
pub use mem::{MemMonitor, MemStats};
pub use metrics::{Delta, Level, Metrics, MetricsCollector, Sparkline, TrackedMetric};
pub use swap::{SwapMonitor, SwapStats};
