use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{Diagnosis, ProbeOutcome, ServiceState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub state: ServiceState,
    #[serde(with = "time::serde::rfc3339::option")]
    pub checked_at: Option<OffsetDateTime>,
    pub latency_milliseconds: Option<u64>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_ok_at: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    #[serde(default)]
    pub diagnosis: Option<Diagnosis>,
    #[serde(with = "time::serde::rfc3339")]
    pub since: OffsetDateTime,
}

impl ServiceStatus {
    pub fn unknown(since: OffsetDateTime) -> ServiceStatus {
        ServiceStatus {
            state: ServiceState::Unknown,
            checked_at: None,
            latency_milliseconds: None,
            last_ok_at: None,
            last_error: None,
            diagnosis: None,
            since,
        }
    }

    pub fn after(&self, outcome: ProbeOutcome, now: OffsetDateTime) -> ServiceStatus {
        let since = if outcome.state == self.state {
            self.since
        } else {
            now
        };
        let last_ok_at = if outcome.state.answered() {
            Some(now)
        } else {
            self.last_ok_at
        };
        ServiceStatus {
            state: outcome.state,
            checked_at: Some(now),
            latency_milliseconds: outcome.latency_milliseconds,
            last_ok_at,
            last_error: outcome.error,
            diagnosis: outcome.diagnosis,
            since,
        }
    }
}
