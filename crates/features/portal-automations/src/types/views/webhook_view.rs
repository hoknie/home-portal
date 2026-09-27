use crate::types::{Reception, Webhook};

#[derive(Debug, Clone)]
pub struct WebhookView {
    pub webhook: Webhook,
    pub last: Option<Reception>,
}
