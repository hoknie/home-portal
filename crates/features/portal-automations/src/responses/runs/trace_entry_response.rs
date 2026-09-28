use portal_feature::PortalEvent;
use serde::Serialize;

use crate::types::TraceEntry;

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
}

impl TraceEntryResponse {
    pub fn of(entry: &TraceEntry) -> TraceEntryResponse {
        TraceEntryResponse {
            path: entry.path.clone(),
            step: entry.step.clone(),
            label: entry.label.clone(),
            kind: entry.kind.clone(),
            iteration: entry.iteration,
            outcome: entry.outcome.name().to_string(),
            started_at: PortalEvent::timestamp(entry.started_at),
            duration_milliseconds: entry.duration.as_millis() as u64,
            detail: entry.detail.clone(),
            output: entry.output.clone(),
            shape: entry.shape.clone(),
        }
    }
}
