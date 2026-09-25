use serde::{Deserialize, Serialize};

use crate::types::Uptime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UptimeResponse {
    pub range: String,
    pub ratio: Option<f64>,
    pub covered_seconds: u64,
}

impl UptimeResponse {
    pub fn of(uptime: &Uptime) -> UptimeResponse {
        UptimeResponse {
            range: uptime.range.name().to_string(),
            ratio: uptime.ratio,
            covered_seconds: uptime.covered_seconds,
        }
    }
}
