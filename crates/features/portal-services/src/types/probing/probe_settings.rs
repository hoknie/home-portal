use serde::{Deserialize, Serialize};

use super::ProbeKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProbeSettings {
    pub enabled: bool,
    pub kind: ProbeKind,
    pub environment: Option<String>,
    pub path: String,
    pub port: Option<u16>,
    pub every_seconds: u64,
    pub timeout_seconds: u64,
    pub degraded_after_milliseconds: u64,
}

impl ProbeSettings {
    pub const DEFAULT_PATH: &'static str = "/";
    pub const DEFAULT_EVERY_SECONDS: u64 = 30;
    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 5;
    pub const DEFAULT_DEGRADED_AFTER_MILLISECONDS: u64 = 1500;
    pub const EVERY_SECONDS: std::ops::RangeInclusive<u64> = 5..=3600;
    pub const TIMEOUT_SECONDS: std::ops::RangeInclusive<u64> = 1..=60;
    pub const DEGRADED_AFTER_MILLISECONDS: std::ops::RangeInclusive<u64> = 1..=60_000;
}

impl Default for ProbeSettings {
    fn default() -> ProbeSettings {
        ProbeSettings {
            enabled: true,
            kind: ProbeKind::Http,
            environment: None,
            path: Self::DEFAULT_PATH.to_string(),
            port: None,
            every_seconds: Self::DEFAULT_EVERY_SECONDS,
            timeout_seconds: Self::DEFAULT_TIMEOUT_SECONDS,
            degraded_after_milliseconds: Self::DEFAULT_DEGRADED_AFTER_MILLISECONDS,
        }
    }
}
