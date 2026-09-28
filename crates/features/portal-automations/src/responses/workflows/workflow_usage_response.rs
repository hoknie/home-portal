use serde::Serialize;

use crate::types::WorkflowUsage;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowUsageResponse {
    pub kind: String,
    pub id: String,
    pub title: String,
}

impl WorkflowUsageResponse {
    pub fn of(usage: &WorkflowUsage) -> WorkflowUsageResponse {
        WorkflowUsageResponse {
            kind: usage.kind.to_string(),
            id: usage.id.clone(),
            title: usage.title.clone(),
        }
    }
}
