use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::{AutomationSettings, RawAutomation};
use crate::types::RawWebhook;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AutomationsSection {
    #[serde(default)]
    pub automations: Vec<RawAutomation>,
    #[serde(default)]
    pub automation_settings: AutomationSettings,
    #[serde(default)]
    pub webhooks: Vec<RawWebhook>,
}

impl AutomationsSection {
    pub const SECTION: &'static str = "automations";
    pub const SETTINGS: &'static str = "automation_settings";
    pub const WEBHOOKS: &'static str = "webhooks";

    pub fn read(document: &DocumentMut) -> Result<AutomationsSection, String> {
        deserialize_section(document)
    }
}
