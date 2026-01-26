use sysinfo::Disks;

#[derive(Clone)]
pub struct DiskStats {
    pub free_gb: u64,
    pub total_gb: u64,
}

pub struct DiskMonitor {
    disks: Disks,
}

impl DiskMonitor {
    pub fn new() -> Self {
        let disks = Disks::new_with_refreshed_list();
        Self { disks }
    }

    pub fn refresh(&mut self) -> DiskStats {
        self.disks.refresh(true);

        // Find root disk (mount point "/")
        let root = self.disks.iter().find(|d| d.mount_point() == std::path::Path::new("/"));

        match root {
            Some(disk) => DiskStats {
                free_gb: disk.available_space() / 1_073_741_824,
                total_gb: disk.total_space() / 1_073_741_824,
            },
            None => DiskStats {
                free_gb: 0,
                total_gb: 0,
            },
        }
    }
}

impl Default for DiskMonitor {
    fn default() -> Self {
        Self::new()
    }
}
