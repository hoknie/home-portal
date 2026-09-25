use serde::{Deserialize, Serialize};

use super::{DiskReading, Usage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostReading {
    pub hostname: Option<String>,
    pub cpu_percent: f64,
    pub load_average: Option<[f64; 3]>,
    pub memory: Usage,
    pub swap: Usage,
    pub disks: Vec<DiskReading>,
    pub uptime_seconds: u64,
}
