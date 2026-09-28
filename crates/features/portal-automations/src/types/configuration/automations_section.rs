use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::{AutomationSettings, RawAutomation};
use crate::types::{RawServiceId, RawWebhook, RawWorkflow};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AutomationsSection {
    #[serde(default)]
    pub automations: Vec<RawAutomation>,
    #[serde(default)]
    pub automation_settings: AutomationSettings,
    #[serde(default)]
    pub webhooks: Vec<RawWebhook>,
    #[serde(default)]
    pub workflows: Vec<RawWorkflow>,
    #[serde(default)]
    pub services: Option<Vec<RawServiceId>>,
}

impl AutomationsSection {
    pub const SECTION: &'static str = "automations";
    pub const SETTINGS: &'static str = "automation_settings";
    pub const WEBHOOKS: &'static str = "webhooks";

    pub fn service_ids(&self) -> Option<std::collections::BTreeSet<String>> {
        self.services
            .as_ref()
            .map(|services| services.iter().map(|service| service.id.clone()).collect())
    }

    pub fn read(document: &DocumentMut) -> Result<AutomationsSection, String> {
        deserialize_section(document)
    }
}
