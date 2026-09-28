#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowUsage {
    pub kind: &'static str,
    pub id: String,
    pub title: String,
}

impl WorkflowUsage {
    pub const AUTOMATION: &'static str = "automation";
    pub const WEBHOOK: &'static str = "webhook";
    pub const WORKFLOW: &'static str = "workflow";
}
