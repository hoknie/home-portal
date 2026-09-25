use serde::Serialize;

use super::WebhookResponse;

#[derive(Debug, Clone, Serialize)]
pub struct WebhooksResponse {
    pub webhooks: Vec<WebhookResponse>,
}
