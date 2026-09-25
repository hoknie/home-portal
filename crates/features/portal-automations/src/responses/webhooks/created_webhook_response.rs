use serde::Serialize;

use super::WebhookResponse;

#[derive(Debug, Clone, Serialize)]
pub struct CreatedWebhookResponse {
    pub webhook: WebhookResponse,
    pub token: Option<String>,
}
