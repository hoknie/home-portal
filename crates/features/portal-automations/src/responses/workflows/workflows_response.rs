use schemars::JsonSchema;
use serde::Serialize;

use super::WorkflowResponse;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WorkflowsResponse {
    pub workflows: Vec<WorkflowResponse>,
}
