use super::WorkflowUsage;
use crate::types::{ActiveRun, RawWorkflow, RunRecord, Workflow};

#[derive(Debug, Clone)]
pub struct WorkflowView {
    pub raw: RawWorkflow,
    pub workflow: Workflow,
    pub used_by: Vec<WorkflowUsage>,
    pub last: Option<RunRecord>,
    pub active: Option<ActiveRun>,
}
