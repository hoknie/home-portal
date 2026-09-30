use portal_feature::{EventName, PortalEvent};
use portal_model::ServiceState;

use crate::helpers::body_path;

pub struct Catalogue;

impl Catalogue {
    pub const AUTOMATION_FIELD: &'static str = "automation.id";
    pub const RUN_ID_FIELD: &'static str = "run.id";
    pub const RUN_MANUAL_FIELD: &'static str = "run.manual";
    pub const RUN_BY_FIELD: &'static str = "run.by";
    pub const RUN_FIELDS: [&'static str; 4] = [
        Self::AUTOMATION_FIELD,
        Self::RUN_ID_FIELD,
        Self::RUN_MANUAL_FIELD,
        Self::RUN_BY_FIELD,
    ];

    pub const BODY_FIELD: &'static str = "webhook.body";

    pub fn fields_of(event: EventName) -> Vec<&'static str> {
        let mut fields = Self::carried_by(event);
        if event == EventName::WebhookReceived {
            fields.push(Self::BODY_FIELD);
        }
        fields
    }

    pub fn carried_by(event: EventName) -> Vec<&'static str> {
        let mut fields = vec![PortalEvent::NAME_FIELD, PortalEvent::AT_FIELD];
        fields.extend_from_slice(event.fields());
        fields.extend_from_slice(&Self::RUN_FIELDS);
        fields
    }

    pub fn knows(field: &str) -> bool {
        let short = ["name", "at"].contains(&field);
        let variable = field
            .strip_prefix(PortalEvent::VARIABLE_PREFIX)
            .is_some_and(|name| {
                name.starts_with(|c: char| c.is_ascii_lowercase())
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            });
        short
            || variable
            || body_path(field).is_some()
            || EventName::ALL
                .iter()
                .any(|event| Self::fields_of(*event).contains(&field))
    }

    pub fn states() -> Vec<&'static str> {
        ServiceState::ALL.iter().map(|state| state.name()).collect()
    }

    pub fn event_names() -> Vec<&'static str> {
        EventName::ALL.iter().map(|event| event.name()).collect()
    }
}
