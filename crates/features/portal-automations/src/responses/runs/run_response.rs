use std::collections::BTreeMap;

use portal_feature::PortalEvent;
use serde::Serialize;

use super::{OutcomeResponse, OutputResponse};
use crate::types::{ActiveRun, RunRecord};

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
    pub const QUEUED: &'static str = "queued";
    pub const RUNNING: &'static str = "running";
    pub const STOPPING: &'static str = "stopping";

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

    pub fn active(run: &ActiveRun) -> RunResponse {
        let (started_at, duration) = match run.started {
            Some((at, instant)) => (at, instant.elapsed()),
            None => (run.admitted_at, std::time::Duration::ZERO),
        };
        let (stdout, stderr) = run.control.output();
        let started_at = PortalEvent::timestamp(started_at);
        RunResponse {
            id: run.run_id.to_string(),
            automation: run.automation.clone(),
            event: run
                .fields
                .iter()
                .find(|(key, _)| key == PortalEvent::NAME_FIELD)
                .map(|(_, value)| value.clone())
                .unwrap_or_default(),
            fields: run.fields.iter().cloned().collect(),
            arguments: run.arguments.clone(),
            started_at: started_at.clone(),
            outcome: OutcomeResponse {
                result: if run.running() {
                    Self::RUNNING
                } else {
                    Self::QUEUED
                }
                .to_string(),
                exit_code: None,
                reason: run.stopped_by.as_ref().map(|_| Self::STOPPING.to_string()),
                duration_milliseconds: duration.as_millis() as u64,
                count: 1,
                last_at: started_at,
                stdout: OutputResponse::of(&stdout),
                stderr: OutputResponse::of(&stderr),
            },
        }
    }
}
