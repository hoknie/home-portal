use portal_feature::{EventName, PortalEvent};
use portal_model::ServiceState;

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

    pub fn fields_of(event: EventName) -> Vec<&'static str> {
        let mut fields = vec![PortalEvent::NAME_FIELD, PortalEvent::AT_FIELD];
        fields.extend_from_slice(event.fields());
        fields.extend_from_slice(&Self::RUN_FIELDS);
        fields
    }

    pub fn states() -> Vec<&'static str> {
        ServiceState::ALL.iter().map(|state| state.name()).collect()
    }

    pub fn event_names() -> Vec<&'static str> {
        EventName::ALL.iter().map(|event| event.name()).collect()
    }
}
