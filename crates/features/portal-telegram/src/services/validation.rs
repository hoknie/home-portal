use portal_config::ConfigStore;
use portal_feature::FieldError;
use toml_edit::DocumentMut;

use crate::types::TelegramSection;

pub const SECTION: &str = "notifications.telegram";

pub fn check_telegram(document: &DocumentMut, configuration: &ConfigStore) -> Vec<FieldError> {
    let settings = match TelegramSection::read(document) {
        Ok(settings) => settings,
        Err(message) => return vec![FieldError::new(SECTION, message)],
    };
    if !settings.enabled {
        return Vec::new();
    }
    let mut errors = Vec::new();
    match &settings.secret {
        None => errors.push(FieldError::new(
            format!("{SECTION}.secret"),
            "must name the secret holding the bot token",
        )),
        Some(name) if configuration.secret(name).is_none() => errors.push(FieldError::new(
            format!("{SECTION}.secret"),
            format!("names {name}, which is not set"),
        )),
        Some(_) => {}
    }
    if settings.chat_id.as_deref().is_none_or(str::is_empty) {
        errors.push(FieldError::new(
            format!("{SECTION}.chat_id"),
            "must name the chat the portal writes to",
        ));
    }
    errors
}
