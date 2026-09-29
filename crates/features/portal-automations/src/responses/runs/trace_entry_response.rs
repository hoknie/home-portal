use portal_feature::PortalEvent;
use serde::Serialize;

use super::OutputResponse;
use time::OffsetDateTime;

use crate::types::{StepOutcome, TraceEntry};

#[derive(Debug, Clone, Serialize)]
pub struct TraceEntryResponse {
    pub path: String,
    pub step: String,
    pub label: String,
    pub kind: String,
    pub iteration: Option<usize>,
    pub outcome: String,
    pub started_at: String,
    pub duration_milliseconds: u64,
    pub detail: String,
    pub output: Option<String>,
    pub shape: Option<String>,
    pub stdout: Option<OutputResponse>,
    pub stderr: Option<OutputResponse>,
    pub command: Option<Vec<String>>,
    pub budget_reached: bool,
    pub values: Vec<RenderedResponse>,
    pub log: Vec<String>,
    pub values_dropped: usize,
    pub log_dropped: usize,
    pub item: Option<String>,
    pub level: Option<String>,
    pub wait_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderedResponse {
    pub template: String,
    pub value: String,
}

impl TraceEntryResponse {
    pub fn duration_of(entry: &TraceEntry, now: Option<OffsetDateTime>) -> u64 {
        match now {
            Some(now) if entry.outcome == StepOutcome::Running => {
                (now - entry.started_at).whole_milliseconds().max(0) as u64
            }
            _ => entry.duration.as_millis() as u64,
        }
    }

    pub fn of(entry: &TraceEntry, now: Option<OffsetDateTime>) -> TraceEntryResponse {
        TraceEntryResponse {
            path: entry.path.clone(),
            step: entry.step.clone(),
            label: entry.label.clone(),
            kind: entry.kind.clone(),
            iteration: entry.iteration,
            outcome: entry.outcome.name().to_string(),
            started_at: PortalEvent::timestamp(entry.started_at),
            duration_milliseconds: Self::duration_of(entry, now),
            detail: entry.detail.clone(),
            output: entry.output.clone(),
            shape: entry.shape.clone(),
            stdout: entry
                .streams
                .as_ref()
                .map(|streams| OutputResponse::of(&streams.stdout)),
            stderr: entry
                .streams
                .as_ref()
                .map(|streams| OutputResponse::of(&streams.stderr)),
            command: entry
                .streams
                .as_ref()
                .map(|streams| streams.command.clone()),
            budget_reached: entry
                .streams
                .as_ref()
                .is_some_and(|streams| streams.budget_reached),
            values: entry
                .log
                .values
                .iter()
                .map(|value| RenderedResponse {
                    template: value.template.clone(),
                    value: value.value.clone(),
                })
                .collect(),
            log: entry.log.lines.clone(),
            values_dropped: entry.log.values_dropped,
            log_dropped: entry.log.lines_dropped,
            item: entry.item.clone(),
            level: entry.level.map(|level| level.name().to_string()),
            wait_seconds: entry.wait_seconds,
        }
    }
}
