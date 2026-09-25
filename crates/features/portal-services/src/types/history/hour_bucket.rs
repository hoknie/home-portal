use portal_model::ServiceState;
use serde::{Deserialize, Serialize};

use super::Sample;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HourBucket {
    pub hour: i64,
    pub up: u32,
    pub degraded: u32,
    pub down: u32,
    pub unreadable: u32,
    pub minimum: Option<u32>,
    pub maximum: Option<u32>,
    pub latency_sum: u64,
    pub latency_count: u32,
    pub covered_seconds: u64,
    pub answered_seconds: u64,
}

impl HourBucket {
    pub const SECONDS: i64 = 3600;

    pub fn starting(hour: i64) -> HourBucket {
        HourBucket {
            hour,
            ..HourBucket::default()
        }
    }

    pub fn hour_of(at: i64) -> i64 {
        at - at.rem_euclid(Self::SECONDS)
    }

    pub fn absorb(&mut self, sample: &Sample) {
        match sample.state {
            ServiceState::Up => self.up += 1,
            ServiceState::Degraded => self.degraded += 1,
            ServiceState::Down => self.down += 1,
            ServiceState::Unreadable => self.unreadable += 1,
            ServiceState::Unknown => {}
        }
        if let Some(latency) = sample.latency {
            self.minimum = Some(self.minimum.map_or(latency, |minimum| minimum.min(latency)));
            self.maximum = Some(self.maximum.map_or(latency, |maximum| maximum.max(latency)));
            self.latency_sum += u64::from(latency);
            self.latency_count += 1;
        }
        self.covered_seconds += u64::from(sample.covered);
        self.answered_seconds += sample.answered_seconds();
    }

    pub fn average(&self) -> Option<u32> {
        (self.latency_count > 0).then(|| {
            u32::try_from(self.latency_sum / u64::from(self.latency_count)).unwrap_or(u32::MAX)
        })
    }

    pub fn worst(&self) -> ServiceState {
        if self.unreadable > 0 {
            ServiceState::Unreadable
        } else if self.down > 0 {
            ServiceState::Down
        } else if self.degraded > 0 {
            ServiceState::Degraded
        } else if self.up > 0 {
            ServiceState::Up
        } else {
            ServiceState::Unknown
        }
    }
}
