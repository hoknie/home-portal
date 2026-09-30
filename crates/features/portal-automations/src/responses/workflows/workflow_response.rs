use serde::Serialize;
use serde_json::Value;

use super::{InputResponse, WorkflowUsageResponse};
use crate::responses::RunResponse;
use crate::types::WorkflowView;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub timeout_seconds: u64,
    pub inputs: Vec<InputResponse>,
    pub steps: Value,
    pub steps_version: String,
    pub used_by: Vec<WorkflowUsageResponse>,
    pub last_run: Option<RunResponse>,
    pub active_run: Option<RunResponse>,
}

impl WorkflowResponse {
    pub fn of(view: &WorkflowView) -> WorkflowResponse {
        let workflow = &view.workflow;
        WorkflowResponse {
            id: workflow.id.clone(),
            title: workflow.title.clone(),
            enabled: workflow.enabled,
            description: workflow.description.clone(),
            tags: workflow.tags.clone(),
            timeout_seconds: workflow.timeout_seconds,
            inputs: workflow.inputs.iter().map(InputResponse::of).collect(),
            steps: serde_json::to_value(&view.raw.steps).unwrap_or(Value::Array(Vec::new())),
            steps_version: workflow.version.clone(),
            used_by: view.used_by.iter().map(WorkflowUsageResponse::of).collect(),
            last_run: view
                .last
                .as_ref()
                .map(|run| RunResponse::of(run).summarized()),
            active_run: view
                .active
                .as_ref()
                .map(|run| RunResponse::active(run).summarized()),
        }
    }
}
