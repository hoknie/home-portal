use std::sync::Arc;

use toml_edit::DocumentMut;

use super::{AutomationSink, WebhookBook, decoded, decoded_webhooks};
use crate::types::{Automation, AutomationView, Webhook, WebhookView};

#[derive(Clone)]
pub struct Views {
    pub sink: Arc<AutomationSink>,
    pub webhooks: Arc<WebhookBook>,
}

impl Views {
    pub fn automation(&self, automation: Automation) -> AutomationView {
        AutomationView {
            last: self.sink.journal.last_of(&automation.id),
            active: self.sink.active.of_automation(&automation.id),
            automation,
        }
    }

    pub fn automations(&self, document: &DocumentMut) -> Vec<AutomationView> {
        decoded(document)
            .into_iter()
            .map(|automation| self.automation(automation))
            .collect()
    }

    pub fn webhook(&self, webhook: Webhook) -> WebhookView {
        WebhookView {
            last: self.webhooks.last(&webhook.id),
            webhook,
        }
    }

    pub fn webhooks(&self, document: &DocumentMut) -> Vec<WebhookView> {
        decoded_webhooks(document)
            .into_iter()
            .map(|webhook| self.webhook(webhook))
            .collect()
    }
}
