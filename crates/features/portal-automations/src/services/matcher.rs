use portal_feature::{EventName, PortalEvent};
use portal_model::ServiceState;

use crate::types::{Automation, Filters};

pub fn matching<'a>(automations: &'a [Automation], event: &PortalEvent) -> Vec<&'a Automation> {
    automations
        .iter()
        .filter(|automation| matches(automation, event))
        .collect()
}

fn matches(automation: &Automation, event: &PortalEvent) -> bool {
    if !automation.enabled || automation.trigger.event != event.name {
        return false;
    }
    if let Some(target) = &event.target {
        return *target == automation.id;
    }
    if matches!(event.name, EventName::Schedule | EventName::Manual) {
        return false;
    }
    filters_accept(&automation.trigger.filters, event)
}

fn filters_accept(filters: &Filters, event: &PortalEvent) -> bool {
    let value = |field: &str| event.value(field).unwrap_or_default();
    let listed = |list: &[String], field: &str| {
        list.is_empty() || list.iter().any(|wanted| wanted == value(field))
    };
    if event.name == EventName::ServiceStatusChanged
        && value("status.from") == ServiceState::Unknown.name()
        && !filters.states.from_unknown
    {
        return false;
    }
    listed(&filters.services, "service.id")
        && listed(&filters.states.from, "status.from")
        && listed(&filters.states.to, "status.to")
        && listed(&filters.users, "user.name")
        && listed(&filters.environments, "client.environment")
        && listed(&filters.webhooks, "webhook.id")
}
