use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::StoredTail;
use crate::types::{LogLevel, Rendered, StepLog, StepOutcome, Streams, TraceEntry};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<StoredTail>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<StoredTail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub budget_reached: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log: Vec<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub values_dropped: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub log_dropped: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_seconds: Option<u64>,
}

fn is_zero(count: &usize) -> bool {
    *count == 0
}

fn is_false(flag: &bool) -> bool {
    !*flag
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
            stdout: entry
                .streams
                .as_ref()
                .map(|streams| StoredTail::of(&streams.stdout)),
            stderr: entry
                .streams
                .as_ref()
                .map(|streams| StoredTail::of(&streams.stderr)),
            command: entry
                .streams
                .as_ref()
                .map(|streams| streams.command.clone())
                .unwrap_or_default(),
            budget_reached: entry
                .streams
                .as_ref()
                .is_some_and(|streams| streams.budget_reached),
            values: entry
                .log
                .values
                .iter()
                .map(|value| (value.template.clone(), value.value.clone()))
                .collect(),
            log: entry.log.lines.clone(),
            values_dropped: entry.log.values_dropped,
            log_dropped: entry.log.lines_dropped,
            item: entry.item.clone(),
            level: entry.level.map(|level| level.name().to_string()),
            wait_seconds: entry.wait_seconds,
        }
    }

    pub fn into_entry(self) -> Option<TraceEntry> {
        let streams = (self.stdout.is_some() || self.stderr.is_some()).then(|| Streams {
            stdout: self.stdout.map(StoredTail::into_tail).unwrap_or_default(),
            stderr: self.stderr.map(StoredTail::into_tail).unwrap_or_default(),
            command: self.command,
            budget_reached: self.budget_reached,
        });
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
            streams,
            log: StepLog {
                values: self
                    .values
                    .into_iter()
                    .map(|(template, value)| Rendered { template, value })
                    .collect(),
                lines: self.log,
                values_dropped: self.values_dropped,
                lines_dropped: self.log_dropped,
            },
            item: self.item,
            level: self.level.as_deref().and_then(LogLevel::of),
            wait_seconds: self.wait_seconds,
        })
    }
}
