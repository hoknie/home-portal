use serde::Serialize;

use crate::responses::RunSettingsResponse;
use crate::types::{Webhook, WebhookAction};

#[derive(Debug, Clone, Serialize)]
pub struct WebhookActionResponse {
    pub variables: Vec<String>,
    pub action: String,
    pub run: Option<RunSettingsResponse>,
}

impl WebhookActionResponse {
    pub fn of(webhook: &Webhook) -> WebhookActionResponse {
        WebhookActionResponse {
            variables: webhook.variables.clone(),
            action: webhook.action.name().to_string(),
            run: match &webhook.action {
                WebhookAction::Event => None,
                WebhookAction::Script(run) => Some(RunSettingsResponse::of(run)),
            },
        }
    }
}
