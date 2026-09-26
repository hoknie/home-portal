use time::OffsetDateTime;

use super::{Outcome, Seen, SkipReason};
use crate::types::{Finished, Invocation, Pending, Tail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub id: u64,
    pub automation: String,
    pub fields: Vec<(String, String)>,
    pub arguments: Vec<String>,
    pub seen: Seen,
    pub result: Finished,
}

impl RunRecord {
    pub const STOPPED: &'static str = "stopped";

    pub fn finished(
        pending: &Pending,
        arguments: Vec<String>,
        started_at: OffsetDateTime,
        result: Finished,
    ) -> RunRecord {
        RunRecord {
            id: pending.run_id,
            automation: pending.automation.id.clone(),
            fields: Invocation::fields_of(pending),
            arguments,
            seen: Seen::once(started_at),
            result,
        }
    }

    pub fn skipped(pending: &Pending, reason: SkipReason, at: OffsetDateTime) -> RunRecord {
        Self::finished(
            pending,
            Vec::new(),
            at,
            Finished {
                outcome: Outcome::Skipped,
                exit_code: None,
                reason: Some(reason.name().to_string()),
                duration: std::time::Duration::ZERO,
                stdout: Tail::default(),
                stderr: Tail::default(),
            },
        )
    }

    pub fn stopped(pending: &Pending, by: &str, at: OffsetDateTime) -> RunRecord {
        Self::finished(
            pending,
            Vec::new(),
            at,
            Finished {
                outcome: Outcome::Stopped,
                exit_code: None,
                reason: Some(Self::stopped_reason(by)),
                duration: std::time::Duration::ZERO,
                stdout: Tail::default(),
                stderr: Tail::default(),
            },
        )
    }

    pub fn stopped_reason(by: &str) -> String {
        if by.is_empty() {
            Self::STOPPED.to_string()
        } else {
            format!("{} by {by}", Self::STOPPED)
        }
    }

    pub fn event(&self) -> &str {
        self.fields
            .iter()
            .find(|(key, _)| key == portal_feature::PortalEvent::NAME_FIELD)
            .map(|(_, value)| value.as_str())
            .unwrap_or_default()
    }

    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn merges_with(&self, other: &RunRecord) -> bool {
        self.automation == other.automation
            && self.result.outcome == Outcome::Skipped
            && other.result.outcome == Outcome::Skipped
            && self.result.reason == other.result.reason
            && self.event() == other.event()
            && self.field("webhook.id") == other.field("webhook.id")
    }
}
