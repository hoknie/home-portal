use crate::types::{RunSettings, WorkflowCall};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookAction {
    Event,
    Script(RunSettings),
    Workflow(WorkflowCall),
}

impl WebhookAction {
    pub const EVENT: &'static str = "event";
    pub const SCRIPT: &'static str = "script";
    pub const WORKFLOW: &'static str = "workflow";

    pub fn name(&self) -> &'static str {
        match self {
            WebhookAction::Event => Self::EVENT,
            WebhookAction::Script(_) | WebhookAction::Workflow(_) => Self::SCRIPT,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            WebhookAction::Event => Self::EVENT,
            WebhookAction::Script(_) => Self::SCRIPT,
            WebhookAction::Workflow(_) => Self::WORKFLOW,
        }
    }
}
