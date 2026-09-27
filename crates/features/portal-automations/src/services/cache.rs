use std::sync::{Arc, PoisonError, RwLock};

use jiff::tz::TimeZone;
use portal_feature::{Module, ModuleSwitches};
use tokio::sync::Notify;
use toml_edit::DocumentMut;

use super::{decoded, decoded_webhooks};
use crate::types::{Automation, AutomationsSection, Webhook};

pub struct AutomationCache {
    automations: RwLock<Arc<Vec<Automation>>>,
    webhooks: RwLock<Arc<Vec<Webhook>>>,
    zone: RwLock<TimeZone>,
    switches: RwLock<ModuleSwitches>,
    changed: Notify,
}

impl AutomationCache {
    pub fn of(document: &DocumentMut) -> AutomationCache {
        let cache = AutomationCache {
            automations: RwLock::new(Arc::new(Vec::new())),
            webhooks: RwLock::new(Arc::new(Vec::new())),
            zone: RwLock::new(TimeZone::UTC),
            switches: RwLock::new(ModuleSwitches::default()),
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
        *self
            .switches
            .write()
            .unwrap_or_else(PoisonError::into_inner) =
            ModuleSwitches::resolve(document).unwrap_or_default();
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

    pub fn switches(&self) -> ModuleSwitches {
        *self.switches.read().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn automations_on(&self) -> bool {
        self.switches().is_on(Module::Automations)
    }

    pub fn webhooks_on(&self) -> bool {
        self.switches().is_on(Module::Webhooks) && self.automations_on()
    }

    pub fn runnable(&self, id: &str) -> bool {
        if !self.automations_on() {
            return false;
        }
        self.find(id).is_some_and(|automation| automation.enabled)
            || (self.webhooks_on()
                && self
                    .webhook(id)
                    .is_some_and(|webhook| webhook.enabled && webhook.as_automation().is_some()))
    }

    pub fn find(&self, id: &str) -> Option<Automation> {
        self.automations()
            .iter()
            .find(|automation| automation.id == id)
            .cloned()
    }
}
