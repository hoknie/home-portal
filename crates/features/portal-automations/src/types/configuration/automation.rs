use portal_feature::FieldError;

use super::{RawAutomation, RunSettings, Trigger};
use crate::helpers::{check_tags, unknown_placeholders};
use crate::types::Catalogue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Automation {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub tags: Vec<String>,
    pub cooldown_seconds: u64,
    pub trigger: Trigger,
    pub run: RunSettings,
}

impl Automation {
    pub const LONGEST_ID: usize = 63;
    pub const RESERVED_IDS: [&'static str; 4] = ["runs", "catalogue", "scripts", "schedule"];

    pub fn decode(raw: &RawAutomation) -> Result<Automation, Vec<FieldError>> {
        let mut errors = Vec::new();
        if Self::RESERVED_IDS.contains(&raw.id.as_str()) {
            errors.push(FieldError::new(
                "id",
                format!(
                    "is reserved by the interface; {} cannot be used",
                    Self::RESERVED_IDS.join(", ")
                ),
            ));
        } else if !Self::valid_id(&raw.id) {
            errors.push(FieldError::new(
                "id",
                format!(
                    "must be 1 to {} lowercase letters, digits and -",
                    Self::LONGEST_ID
                ),
            ));
        }
        if raw.title.trim().is_empty() {
            errors.push(FieldError::new("title", "must not be empty"));
        }
        errors.extend(check_tags(&raw.tags));
        let cooldown = raw.cooldown_seconds.unwrap_or(0);
        if cooldown < 0 {
            errors.push(FieldError::new("cooldown_seconds", "must not be negative"));
        }
        let trigger = Trigger::decode(&raw.when).map_err(|found| errors.extend(found));
        let run = RunSettings::decode(&raw.run).map_err(|found| errors.extend(found));
        if let Ok(trigger) = &trigger {
            let allowed = Catalogue::fields_of(trigger.event);
            let webhook = trigger.event == portal_feature::EventName::WebhookReceived;
            for (index, argument) in raw.run.args.iter().enumerate() {
                let unknown: Vec<String> = unknown_placeholders(argument, &allowed)
                    .into_iter()
                    .filter(|name| {
                        !(webhook && name.starts_with(portal_feature::PortalEvent::VARIABLE_PREFIX))
                    })
                    .collect();
                if let Some(name) = unknown.first() {
                    errors.push(FieldError::new(
                        format!("run.args[{index}]"),
                        format!(
                            "names {{{{{name}}}}}, which is not a field of {}; its fields are {}",
                            trigger.event.name(),
                            allowed.join(", ")
                        ),
                    ));
                }
            }
        }
        match (trigger, run) {
            (Ok(trigger), Ok(run)) if errors.is_empty() => Ok(Automation {
                id: raw.id.clone(),
                title: raw.title.clone(),
                enabled: raw.enabled.unwrap_or(true),
                tags: raw.tags.clone(),
                cooldown_seconds: cooldown as u64,
                trigger,
                run,
            }),
            _ => Err(errors),
        }
    }

    pub fn valid_id(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= Self::LONGEST_ID
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }
}
