use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::{EventName, StatusChange, Visitor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalEvent {
    pub name: EventName,
    pub at: OffsetDateTime,
    pub fields: Vec<(&'static str, String)>,
    pub target: Option<String>,
    pub variables: Vec<(String, String)>,
}

impl PortalEvent {
    pub const NAME_FIELD: &'static str = "event.name";
    pub const AT_FIELD: &'static str = "event.at";
    pub const VALUE_LIMIT: usize = 1024;
    pub const THROTTLED: &'static str = "throttled";
    pub const CREDENTIALS: &'static str = "credentials";

    pub fn of(name: EventName, at: OffsetDateTime, values: &[(&str, &str)]) -> PortalEvent {
        let mut fields = vec![
            (Self::NAME_FIELD, name.name().to_string()),
            (Self::AT_FIELD, Self::timestamp(at)),
        ];
        for field in name.fields() {
            let value = values
                .iter()
                .find(|(key, _)| key == field)
                .map(|(_, value)| Self::clean(value))
                .unwrap_or_default();
            fields.push((*field, value));
        }
        PortalEvent {
            name,
            at,
            fields,
            target: None,
            variables: Vec::new(),
        }
    }

    pub const VARIABLE_PREFIX: &'static str = "webhook.";

    pub fn with_variables(mut self, variables: &[(String, String)]) -> PortalEvent {
        self.variables = variables
            .iter()
            .map(|(name, value)| {
                (
                    format!("{}{name}", Self::VARIABLE_PREFIX),
                    Self::clean(value),
                )
            })
            .collect();
        self
    }

    pub fn aimed_at(mut self, automation: impl Into<String>) -> PortalEvent {
        self.target = Some(automation.into());
        self
    }

    pub fn portal(name: EventName, address: &str, at: OffsetDateTime) -> PortalEvent {
        Self::of(
            name,
            at,
            &[
                ("portal.address", address),
                ("portal.version", env!("CARGO_PKG_VERSION")),
            ],
        )
    }

    pub fn status_changed(change: &StatusChange, at: OffsetDateTime) -> PortalEvent {
        Self::of(
            EventName::ServiceStatusChanged,
            at,
            &[
                ("service.id", &change.service),
                ("service.name", &change.name),
                ("status.from", &change.was),
                ("status.to", &change.now),
                ("status.error", change.error.as_deref().unwrap_or_default()),
                (
                    "status.diagnosis",
                    change.diagnosis.as_deref().unwrap_or_default(),
                ),
            ],
        )
    }

    pub fn visited(name: EventName, visitor: &Visitor, at: OffsetDateTime) -> PortalEvent {
        Self::of(
            name,
            at,
            &[
                ("user.name", &visitor.user),
                ("client.address", &visitor.address),
                ("client.environment", &visitor.environment),
                ("sign_in.reason", visitor.reason.unwrap_or_default()),
            ],
        )
    }

    pub fn value(&self, field: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| *key == field)
            .map(|(_, value)| value.as_str())
    }

    pub fn timestamp(at: OffsetDateTime) -> String {
        at.format(&Rfc3339).unwrap_or_default()
    }

    pub fn clean(value: &str) -> String {
        let mut cleaned = String::with_capacity(value.len().min(Self::VALUE_LIMIT));
        for character in value.chars().filter(|character| *character != '\0') {
            if cleaned.len() + character.len_utf8() > Self::VALUE_LIMIT {
                break;
            }
            cleaned.push(character);
        }
        cleaned
    }
}
