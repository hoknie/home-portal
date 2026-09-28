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

    pub fn name(&self) -> &'static str {
        match self {
            WebhookAction::Event => Self::EVENT,
            WebhookAction::Script(_) | WebhookAction::Workflow(_) => Self::SCRIPT,
        }
    }
}
