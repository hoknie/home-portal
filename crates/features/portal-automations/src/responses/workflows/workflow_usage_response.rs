use schemars::JsonSchema;
use serde::Serialize;

use crate::types::WorkflowUsage;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WorkflowUsageResponse {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub variables: Vec<String>,
}

impl WorkflowUsageResponse {
    pub fn of(usage: &WorkflowUsage) -> WorkflowUsageResponse {
        WorkflowUsageResponse {
            kind: usage.kind.to_string(),
            id: usage.id.clone(),
            title: usage.title.clone(),
            variables: usage.variables.clone(),
        }
    }
}
