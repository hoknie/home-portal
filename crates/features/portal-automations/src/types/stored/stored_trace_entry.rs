use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::{StepOutcome, TraceEntry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTraceEntry {
    pub path: String,
    pub step: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub iteration: Option<usize>,
    pub outcome: String,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
    pub duration_milliseconds: u64,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
}

impl StoredTraceEntry {
    pub fn of(entry: &TraceEntry) -> StoredTraceEntry {
        StoredTraceEntry {
            path: entry.path.clone(),
            step: entry.step.clone(),
            label: entry.label.clone(),
            kind: entry.kind.clone(),
            iteration: entry.iteration,
            outcome: entry.outcome.name().to_string(),
            started_at: entry.started_at,
            duration_milliseconds: entry.duration.as_millis() as u64,
            detail: entry.detail.clone(),
            output: entry.output.clone(),
            shape: entry.shape.clone(),
        }
    }

    pub fn into_entry(self) -> Option<TraceEntry> {
        Some(TraceEntry {
            path: self.path,
            step: self.step,
            label: self.label,
            kind: self.kind,
            iteration: self.iteration,
            outcome: StepOutcome::of(&self.outcome)?,
            started_at: self.started_at,
            duration: Duration::from_millis(self.duration_milliseconds),
            detail: self.detail,
            output: self.output,
            shape: self.shape,
        })
    }
}
