use schemars::JsonSchema;
use serde::Serialize;

use super::WebhookResponse;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CreatedWebhookResponse {
    pub webhook: WebhookResponse,
    pub token: Option<String>,
}
