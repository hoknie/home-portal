use std::sync::{Arc, PoisonError, RwLock};

use jiff::tz::TimeZone;
use tokio::sync::Notify;
use toml_edit::DocumentMut;

use super::{decoded, decoded_webhooks};
use crate::types::{Automation, AutomationsSection, Webhook};

pub struct AutomationCache {
    automations: RwLock<Arc<Vec<Automation>>>,
    webhooks: RwLock<Arc<Vec<Webhook>>>,
    zone: RwLock<TimeZone>,
    changed: Notify,
}

impl AutomationCache {
    pub fn of(document: &DocumentMut) -> AutomationCache {
        let cache = AutomationCache {
            automations: RwLock::new(Arc::new(Vec::new())),
            webhooks: RwLock::new(Arc::new(Vec::new())),
            zone: RwLock::new(TimeZone::UTC),
            changed: Notify::new(),
        };
        cache.refresh(document);
        cache
    }

    pub fn refresh(&self, document: &DocumentMut) {
        let zone = AutomationsSection::read(document)
            .ok()
            .and_then(|section| section.automation_settings.zone().ok())
            .unwrap_or_else(TimeZone::system);
        *self
            .automations
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Arc::new(decoded(document));
        *self
            .webhooks
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Arc::new(decoded_webhooks(document));
        *self.zone.write().unwrap_or_else(PoisonError::into_inner) = zone;
        self.changed.notify_waiters();
    }

    pub async fn changed(&self) {
        self.changed.notified().await;
    }

    pub fn automations(&self) -> Arc<Vec<Automation>> {
        self.automations
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn zone(&self) -> TimeZone {
        self.zone
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .automations()
            .iter()
            .flat_map(|automation| automation.tags.clone())
            .chain(
                self.webhooks()
                    .iter()
                    .flat_map(|webhook| webhook.tags.clone()),
            )
            .collect();
        tags.sort_by_key(|tag| tag.to_lowercase());
        tags.dedup_by(|left, right| left.to_lowercase() == right.to_lowercase());
        tags
    }

    pub fn webhooks(&self) -> Arc<Vec<Webhook>> {
        self.webhooks
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn webhook(&self, id: &str) -> Option<Webhook> {
        self.webhooks()
            .iter()
            .find(|webhook| webhook.id == id)
            .cloned()
    }

    pub fn runnable(&self, id: &str) -> bool {
        self.find(id).is_some_and(|automation| automation.enabled)
            || self
                .webhook(id)
                .is_some_and(|webhook| webhook.enabled && webhook.as_automation().is_some())
    }

    pub fn find(&self, id: &str) -> Option<Automation> {
        self.automations()
            .iter()
            .find(|automation| automation.id == id)
            .cloned()
    }
}
