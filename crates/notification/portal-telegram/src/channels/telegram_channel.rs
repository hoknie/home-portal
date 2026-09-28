use async_trait::async_trait;
use portal_feature::{Channel, ChannelReadiness, FieldError, Notification, SecretSource};
use serde_json::{Value, json};
use toml_edit::{DocumentMut, Table};

use crate::clients::Bot;
use crate::helpers::put;
use crate::types::TelegramSettings;

pub struct TelegramChannel {
    bot: Bot,
}

impl TelegramChannel {
    pub const NAME: &'static str = "telegram";
    pub const NOT_CONFIGURED: &'static str =
        "Telegram is not configured; set [notifications.telegram] with enabled, secret and chat_id";
    pub const SECRET_RULE: &'static str = "must name the secret holding the bot token";
    pub const CHAT_RULE: &'static str = "must name the chat the portal writes to";

    pub fn new(endpoint: &str) -> Result<TelegramChannel, String> {
        Ok(TelegramChannel {
            bot: Bot::new(endpoint)?,
        })
    }

    fn field(name: &str) -> String {
        format!("{}.{name}", TelegramSettings::SECTION)
    }

    fn missing(settings: &TelegramSettings, secrets: &dyn SecretSource) -> Vec<FieldError> {
        let mut errors = Vec::new();
        match &settings.secret {
            None => errors.push(FieldError::new(Self::field("secret"), Self::SECRET_RULE)),
            Some(name) if !secrets.is_set(name) => errors.push(FieldError::new(
                Self::field("secret"),
                format!("names {name}, which is not set"),
            )),
            Some(_) => {}
        }
        if settings.chat_id.as_deref().is_none_or(str::is_empty) {
            errors.push(FieldError::new(Self::field("chat_id"), Self::CHAT_RULE));
        }
        errors
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn problems(&self, document: &DocumentMut, secrets: &dyn SecretSource) -> Vec<FieldError> {
        match TelegramSettings::read(document) {
            Err(message) => vec![FieldError::new(TelegramSettings::SECTION, message)],
            Ok(settings) if settings.enabled => Self::missing(&settings, secrets),
            Ok(_) => Vec::new(),
        }
    }

    fn readiness(&self, document: &DocumentMut, secrets: &dyn SecretSource) -> ChannelReadiness {
        let Ok(settings) = TelegramSettings::read(document) else {
            return ChannelReadiness::Disabled;
        };
        if !settings.enabled {
            return ChannelReadiness::Disabled;
        }
        match Self::missing(&settings, secrets).into_iter().next() {
            Some(error) => ChannelReadiness::Missing {
                field: error.field,
                message: error.message,
            },
            None => ChannelReadiness::Ready,
        }
    }

    fn settings(&self, document: &DocumentMut) -> Value {
        let settings = TelegramSettings::read(document).unwrap_or_default();
        json!({ "enabled": settings.enabled, "secret": settings.secret, "chat_id": settings.chat_id })
    }

    fn apply(&self, table: &mut Table, settings: &Value) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        let text = |key: &str, errors: &mut Vec<FieldError>| match settings.get(key) {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) if text.trim().is_empty() => None,
            Some(Value::String(text)) => Some(text.trim().to_string()),
            Some(_) => {
                errors.push(FieldError::new(key, "must be text"));
                None
            }
        };
        let enabled = match settings.get("enabled") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(enabled)) => *enabled,
            Some(_) => {
                errors.push(FieldError::new("enabled", "must be true or false"));
                false
            }
        };
        let secret = text("secret", &mut errors);
        let chat_id = text("chat_id", &mut errors);
        if enabled && secret.is_none() {
            errors.push(FieldError::new("secret", Self::SECRET_RULE));
        }
        if enabled && chat_id.is_none() {
            errors.push(FieldError::new("chat_id", Self::CHAT_RULE));
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        put(table, "enabled", Some(enabled.into()));
        put(table, "secret", secret.map(Into::into));
        put(table, "chat_id", chat_id.map(Into::into));
        Ok(())
    }

    async fn deliver(
        &self,
        notification: &Notification,
        document: &DocumentMut,
        secrets: &dyn SecretSource,
    ) -> Result<(), String> {
        let settings = TelegramSettings::read(document)?;
        let chat_id = settings
            .chat_id
            .clone()
            .filter(|_| settings.enabled)
            .ok_or_else(|| Self::NOT_CONFIGURED.to_string())?;
        let token = settings
            .secret
            .as_deref()
            .and_then(|name| secrets.reveal(name))
            .ok_or_else(|| Self::NOT_CONFIGURED.to_string())?;
        self.bot
            .send(&token, &chat_id, &notification.message())
            .await
    }
}
