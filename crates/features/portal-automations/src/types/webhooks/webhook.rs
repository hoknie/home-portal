use portal_feature::{EventName, FieldError, PortalEvent};

use super::{RawWebhook, WebhookAction};
use crate::helpers::{check_tags, unknown_placeholders};
use crate::types::{Automation, Catalogue, Filters, RunSettings, Trigger};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Webhook {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub variables: Vec<String>,
    pub action: WebhookAction,
    pub token_sha256: Option<String>,
}

impl Webhook {
    pub const ADDRESS_PREFIX: &'static str = "/webhook/";
    pub const LONGEST_VARIABLE: usize = 63;
    pub const RESERVED_VARIABLES: [&'static str; 2] = ["id", "title"];

    pub fn decode(raw: &RawWebhook) -> Result<Webhook, Vec<FieldError>> {
        let mut errors = Vec::new();
        if !Self::valid_id(&raw.id) {
            errors.push(FieldError::new("id", "must be a lowercase UUID"));
        }
        if raw.title.trim().is_empty() {
            errors.push(FieldError::new("title", "must not be empty"));
        }
        errors.extend(check_tags(&raw.marks.tags));
        let mut seen = Vec::new();
        for (index, name) in raw.variables.iter().enumerate() {
            if !Self::valid_variable(name) {
                errors.push(FieldError::new(
                    format!("variables[{index}]"),
                    "must start with a lowercase letter and hold only lowercase letters, digits and _",
                ));
            } else if Self::RESERVED_VARIABLES.contains(&name.as_str()) {
                errors.push(FieldError::new(
                    format!("variables[{index}]"),
                    "is the name of a field every webhook has; choose another",
                ));
            } else if seen.contains(&name) {
                errors.push(FieldError::new(
                    format!("variables[{index}]"),
                    "is named twice",
                ));
            }
            seen.push(name);
        }
        if let Some(hash) = &raw.token_sha256
            && !(hash.len() == 64
                && hash
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)))
        {
            errors.push(FieldError::new(
                "token_sha256",
                "must be a SHA-256 in lowercase hex",
            ));
        }
        let action = Self::decode_action(raw, &mut errors);
        match action {
            Some(action) if errors.is_empty() => Ok(Webhook {
                id: raw.id.clone(),
                title: raw.title.clone(),
                enabled: raw.marks.enabled.unwrap_or(true),
                tags: raw.marks.tags.clone(),
                variables: raw.variables.clone(),
                action,
                token_sha256: raw.token_sha256.clone(),
            }),
            _ => Err(errors),
        }
    }

    fn decode_action(raw: &RawWebhook, errors: &mut Vec<FieldError>) -> Option<WebhookAction> {
        match (raw.action.as_str(), &raw.run) {
            (WebhookAction::EVENT, None) => Some(WebhookAction::Event),
            (WebhookAction::EVENT, Some(_)) => {
                errors.push(FieldError::new("run", "is only for action = \"script\""));
                None
            }
            (WebhookAction::SCRIPT, None) => {
                errors.push(FieldError::new(
                    "run",
                    "is required for action = \"script\"",
                ));
                None
            }
            (WebhookAction::SCRIPT, Some(run)) => match RunSettings::decode(run) {
                Ok(settings) => {
                    let allowed = Self::fields_of(&raw.variables);
                    let allowed: Vec<&str> = allowed.iter().map(String::as_str).collect();
                    for (index, argument) in settings.args.iter().enumerate() {
                        if let Some(name) = unknown_placeholders(argument, &allowed).first() {
                            errors.push(FieldError::new(
                                format!("run.args[{index}]"),
                                format!(
                                    "names {{{{{name}}}}}, which is not a field of this webhook; its fields are {}",
                                    allowed.join(", ")
                                ),
                            ));
                        }
                    }
                    Some(WebhookAction::Script(settings))
                }
                Err(found) => {
                    errors.extend(found);
                    None
                }
            },
            _ => {
                errors.push(FieldError::new("action", "must be event or script"));
                None
            }
        }
    }

    pub fn fields_of(variables: &[String]) -> Vec<String> {
        Catalogue::fields_of(EventName::WebhookReceived)
            .into_iter()
            .map(str::to_string)
            .chain(
                variables
                    .iter()
                    .map(|name| format!("{}{name}", PortalEvent::VARIABLE_PREFIX)),
            )
            .collect()
    }

    pub fn address(&self) -> String {
        format!("{}{}", Self::ADDRESS_PREFIX, self.id)
    }

    pub fn as_automation(&self) -> Option<Automation> {
        let WebhookAction::Script(run) = &self.action else {
            return None;
        };
        Some(Automation {
            id: self.id.clone(),
            title: self.title.clone(),
            enabled: self.enabled,
            tags: self.tags.clone(),
            cooldown_seconds: 0,
            trigger: Trigger {
                event: EventName::WebhookReceived,
                filters: Filters::default(),
            },
            run: run.clone(),
        })
    }

    pub fn valid_id(id: &str) -> bool {
        let parts: Vec<&str> = id.split('-').collect();
        let lengths = [8, 4, 4, 4, 12];
        parts.len() == lengths.len()
            && parts.iter().zip(lengths).all(|(part, length)| {
                part.len() == length
                    && part
                        .chars()
                        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
            })
    }

    pub fn valid_variable(name: &str) -> bool {
        name.len() <= Self::LONGEST_VARIABLE
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }
}
