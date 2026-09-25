use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

use super::TelegramSettings;

#[derive(Debug, Default, Deserialize)]
pub struct TelegramSection {
    #[serde(default)]
    pub notifications: Option<Notifications>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Notifications {
    #[serde(default)]
    pub telegram: Option<TelegramSettings>,
}

impl TelegramSection {
    pub fn read(document: &DocumentMut) -> Result<TelegramSettings, String> {
        let section: TelegramSection = deserialize_section(document)?;
        Ok(section
            .notifications
            .and_then(|notifications| notifications.telegram)
            .unwrap_or_default())
    }
}
