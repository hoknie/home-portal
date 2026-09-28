use std::sync::Arc;

use toml_edit::DocumentMut;

use super::{AutomationSink, WebhookBook, decode_workflow, decoded, decoded_webhooks, users_of};
use crate::types::{
    Automation, AutomationView, AutomationsSection, RawWorkflow, Webhook, WebhookView, WorkflowView,
};

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

    pub fn workflows(&self, document: &DocumentMut) -> Vec<WorkflowView> {
        AutomationsSection::read(document)
            .map(|section| section.workflows)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|raw| self.workflow(raw))
            .collect()
    }

    pub fn workflow(&self, raw: RawWorkflow) -> Option<WorkflowView> {
        let workflow = decode_workflow(&raw).ok()?;
        let cache = &self.sink.cache;
        Some(WorkflowView {
            used_by: users_of(
                &workflow.id,
                (&cache.automations(), &cache.webhooks(), &cache.workflows()),
            ),
            last: self.sink.journal.last_of_workflow(&workflow.id),
            active: self.sink.active.of_workflow(&workflow.id),
            raw,
            workflow,
        })
    }
}
