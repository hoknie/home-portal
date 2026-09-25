use std::collections::BTreeSet;
use std::sync::{Mutex, PoisonError};

use sysinfo::{Disks, MemoryRefreshKind, RefreshKind, System};

use crate::types::{DiskReading, HostReading, MetricsSettings, Usage};

pub struct HostReader {
    system: Mutex<System>,
}

impl HostReader {
    pub const PSEUDO_FILE_SYSTEMS: [&'static str; 9] = [
        "tmpfs", "devfs", "devtmpfs", "overlay", "squashfs", "proc", "sysfs", "autofs", "ramfs",
    ];

    pub fn new() -> HostReader {
        let mut system = System::new_with_specifics(Self::wanted());
        system.refresh_cpu_usage();
        HostReader {
            system: Mutex::new(system),
        }
    }

    pub fn read(&self, settings: &MetricsSettings) -> HostReading {
        let mut system = self.system.lock().unwrap_or_else(PoisonError::into_inner);
        system.refresh_specifics(Self::wanted());
        let load = System::load_average();
        HostReading {
            hostname: System::host_name(),
            cpu_percent: f64::from(system.global_cpu_usage()),
            load_average: (load.one + load.five + load.fifteen > 0.0).then_some([
                load.one,
                load.five,
                load.fifteen,
            ]),
            memory: Usage {
                used_bytes: system.used_memory(),
                total_bytes: system.total_memory(),
            },
            swap: Usage {
                used_bytes: system.used_swap(),
                total_bytes: system.total_swap(),
            },
            disks: Self::disks(settings),
            uptime_seconds: System::uptime(),
        }
    }

    fn wanted() -> RefreshKind {
        RefreshKind::nothing()
            .with_cpu(sysinfo::CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::everything())
    }

    fn disks(settings: &MetricsSettings) -> Vec<DiskReading> {
        let disks = Disks::new_with_refreshed_list();
        let mut seen = BTreeSet::new();
        let mut readings: Vec<DiskReading> = disks
            .list()
            .iter()
            .filter(|disk| !Self::is_pseudo(&disk.file_system().to_string_lossy()))
            .filter(|disk| disk.total_space() > 0)
            .filter(|disk| seen.insert(disk.name().to_string_lossy().to_string()))
            .map(|disk| DiskReading {
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
            })
            .collect();
        readings.sort_by(|left, right| left.mount_point.cmp(&right.mount_point));
        match &settings.disks {
            None => readings,
            Some(wanted) => wanted
                .iter()
                .filter_map(|mount| {
                    readings
                        .iter()
                        .find(|reading| &reading.mount_point == mount)
                        .cloned()
                })
                .collect(),
        }
    }

    fn is_pseudo(file_system: &str) -> bool {
        Self::PSEUDO_FILE_SYSTEMS.contains(&file_system.to_ascii_lowercase().as_str())
    }
}

impl Default for HostReader {
    fn default() -> HostReader {
        HostReader::new()
    }
}
