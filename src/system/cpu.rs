use sysinfo::System;

#[derive(Clone)]
pub struct CpuStats {
    pub usage: f32,
    pub cores: usize,
}

pub struct CpuMonitor {
    system: System,
}

impl CpuMonitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        Self { system }
    }

    pub fn cores(&self) -> usize {
        self.system.cpus().len().max(1)
    }

    pub fn refresh(&mut self) -> CpuStats {
        self.system.refresh_cpu_all();

        let cpus = self.system.cpus();
        let total_usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
        let cores = cpus.len();

        CpuStats {
            usage: if cores > 0 { total_usage / cores as f32 } else { 0.0 },
            cores,
        }
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::new()
    }
}
