use super::WebhookView;

#[derive(Debug, Clone)]
pub struct CreatedWebhook {
    pub view: WebhookView,
    pub token: Option<String>,
}
