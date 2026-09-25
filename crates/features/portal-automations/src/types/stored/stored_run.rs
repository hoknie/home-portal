use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{StoredResult, StoredSeen, StoredTail};
use crate::types::{Finished, Outcome, RunRecord, Seen};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredRun {
    pub id: u64,
    pub automation: String,
    pub fields: Vec<(String, String)>,
    pub arguments: Vec<String>,
    pub seen: StoredSeen,
    pub result: StoredResult,
}

impl StoredRun {
    pub fn of(record: &RunRecord) -> StoredRun {
        StoredRun {
            id: record.id,
            automation: record.automation.clone(),
            fields: record.fields.clone(),
            arguments: record.arguments.clone(),
            seen: StoredSeen {
                started_at: record.seen.started_at,
                last_at: record.seen.last_at,
                count: record.seen.count,
            },
            result: StoredResult {
                outcome: record.result.outcome.name().to_string(),
                exit_code: record.result.exit_code,
                reason: record.result.reason.clone(),
                duration_milliseconds: record.result.duration.as_millis() as u64,
                stdout: StoredTail::of(&record.result.stdout),
                stderr: StoredTail::of(&record.result.stderr),
            },
        }
    }

    pub fn into_record(self) -> Option<RunRecord> {
        Some(RunRecord {
            id: self.id,
            automation: self.automation,
            fields: self.fields,
            arguments: self.arguments,
            seen: Seen {
                started_at: self.seen.started_at,
                last_at: self.seen.last_at,
                count: self.seen.count,
            },
            result: Finished {
                outcome: Outcome::of(&self.result.outcome)?,
                exit_code: self.result.exit_code,
                reason: self.result.reason,
                duration: Duration::from_millis(self.result.duration_milliseconds),
                stdout: self.result.stdout.into_tail(),
                stderr: self.result.stderr.into_tail(),
            },
        })
    }
}
