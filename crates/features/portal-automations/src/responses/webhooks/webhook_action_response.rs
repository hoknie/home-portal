use serde::Serialize;

use crate::responses::{RunSettingsResponse, WorkflowCallResponse};
use crate::types::{Webhook, WebhookAction};

#[derive(Debug, Clone, Serialize)]
pub struct WebhookActionResponse {
    pub variables: Vec<String>,
    pub action: String,
    pub run: Option<RunSettingsResponse>,
    pub workflow: Option<WorkflowCallResponse>,
}

impl WebhookActionResponse {
    pub fn of(webhook: &Webhook) -> WebhookActionResponse {
        WebhookActionResponse {
            variables: webhook.variables.clone(),
            action: webhook.action.name().to_string(),
            run: match &webhook.action {
                WebhookAction::Script(run) => Some(RunSettingsResponse::of(run)),
                WebhookAction::Event | WebhookAction::Workflow(_) => None,
            },
            workflow: match &webhook.action {
                WebhookAction::Workflow(call) => Some(WorkflowCallResponse::of(call)),
                WebhookAction::Event | WebhookAction::Script(_) => None,
            },
        }
    }
}
