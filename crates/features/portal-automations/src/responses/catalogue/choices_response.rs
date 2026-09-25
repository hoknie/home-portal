use serde::Serialize;

use super::{ChoiceResponse, WebhookChoiceResponse};

#[derive(Debug, Clone, Serialize)]
pub struct ChoicesResponse {
    pub services: Vec<ChoiceResponse>,
    pub users: Vec<String>,
    pub environments: Vec<String>,
    pub webhooks: Vec<WebhookChoiceResponse>,
    pub tags: Vec<String>,
}
