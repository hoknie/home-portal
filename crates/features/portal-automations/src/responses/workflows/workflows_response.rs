use serde::Serialize;

use super::WorkflowResponse;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowsResponse {
    pub workflows: Vec<WorkflowResponse>,
}
