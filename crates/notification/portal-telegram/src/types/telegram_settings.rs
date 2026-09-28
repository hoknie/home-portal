use portal_config::deserialize_section;
use serde::Deserialize;
use toml_edit::DocumentMut;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Section {
    #[serde(default)]
    notifications: Option<Channels>,
}

#[derive(Debug, Default, Deserialize)]
struct Channels {
    #[serde(default)]
    telegram: Option<TelegramSettings>,
}

impl TelegramSettings {
    pub const SECTION: &'static str = "notifications.telegram";

    pub fn read(document: &DocumentMut) -> Result<TelegramSettings, String> {
        let section: Section = deserialize_section(document)?;
        Ok(section
            .notifications
            .and_then(|channels| channels.telegram)
            .unwrap_or_default())
    }
}
