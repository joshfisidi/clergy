use sysinfo::System;

#[derive(Clone)]
pub struct MemStats {
    pub used_mb: u64,
    pub total_mb: u64,
}

pub struct MemMonitor {
    sys: System,
}

impl MemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        Self { sys }
    }

    pub fn refresh(&mut self) -> MemStats {
        self.sys.refresh_memory();
        MemStats {
            used_mb: self.sys.used_memory() / 1024 / 1024,
            total_mb: self.sys.total_memory() / 1024 / 1024,
        }
    }
}

impl Default for MemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
