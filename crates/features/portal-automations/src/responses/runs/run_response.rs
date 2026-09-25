use std::collections::BTreeMap;

use portal_feature::PortalEvent;
use serde::Serialize;

use super::{OutcomeResponse, OutputResponse};
use crate::types::RunRecord;

#[derive(Debug, Clone, Serialize)]
pub struct RunResponse {
    pub id: String,
    pub automation: String,
    pub event: String,
    pub fields: BTreeMap<String, String>,
    pub arguments: Vec<String>,
    pub started_at: String,
    pub outcome: OutcomeResponse,
}

impl RunResponse {
    pub fn of(record: &RunRecord) -> RunResponse {
        RunResponse {
            id: record.id.to_string(),
            automation: record.automation.clone(),
            event: record.event().to_string(),
            fields: record.fields.iter().cloned().collect(),
            arguments: record.arguments.clone(),
            started_at: PortalEvent::timestamp(record.seen.started_at),
            outcome: OutcomeResponse {
                result: record.result.outcome.name().to_string(),
                exit_code: record.result.exit_code,
                reason: record.result.reason.clone(),
                duration_milliseconds: record.result.duration.as_millis() as u64,
                count: record.seen.count,
                last_at: PortalEvent::timestamp(record.seen.last_at),
                stdout: OutputResponse::of(&record.result.stdout),
                stderr: OutputResponse::of(&record.result.stderr),
            },
        }
    }
}
