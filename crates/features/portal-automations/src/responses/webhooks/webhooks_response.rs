use schemars::JsonSchema;
use serde::Serialize;

use super::WebhookResponse;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct WebhooksResponse {
    pub webhooks: Vec<WebhookResponse>,
}
