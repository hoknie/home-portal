use portal_feature::{EventName, PortalEvent};
use serde_json::{Map, Value};
use time::OffsetDateTime;

use super::sample_of;
use crate::types::Automation;

pub fn manual_event(automation: &Automation, at: OffsetDateTime, read: &[String]) -> PortalEvent {
    let filters = &automation.trigger.filters;
    let first = |list: &[String]| list.first().cloned();
    let chosen = |field: &str| -> Option<String> {
        match field {
            "service.id" => first(&filters.services),
            "status.from" => first(&filters.states.from),
            "status.to" => first(&filters.states.to),
            "user.name" => first(&filters.users),
            "client.environment" => first(&filters.environments),
            "webhook.id" => first(&filters.webhooks),
            "schedule.cron" => filters.cron.as_ref().map(|cron| cron.expression.clone()),
            _ => None,
        }
    };
    let event: EventName = automation.trigger.event;
    let service = chosen("service.id");
    let values: Vec<(&str, String)> = event
        .fields()
        .iter()
        .map(|field| {
            let value = match (*field, &service) {
                ("service.name", Some(id)) => id.clone(),
                _ => chosen(field).unwrap_or_else(|| sample_of(field).to_string()),
            };
            (*field, value)
        })
        .collect();
    let borrowed: Vec<(&str, &str)> = values
        .iter()
        .map(|(field, value)| (*field, value.as_str()))
        .collect();
    let manual = PortalEvent::of(event, at, &borrowed).aimed_at(automation.id.clone());
    let variables: Vec<(String, String)> = automation
        .run
        .args
        .iter()
        .map(String::as_str)
        .chain(
            automation
                .workflow
                .iter()
                .flat_map(|call| call.inputs.iter().filter_map(|(_, value)| value.template())),
        )
        .flat_map(super::placeholders_of)
        .filter_map(|name| name.strip_prefix(PortalEvent::VARIABLE_PREFIX))
        .filter(|name| !["id", "title"].contains(name) && name.split('.').next() != Some("body"))
        .map(str::to_string)
        .chain(read.iter().cloned())
        .collect::<std::collections::BTreeSet<String>>()
        .into_iter()
        .map(|name| (name.clone(), name))
        .collect();
    let body = Value::Object(
        variables
            .iter()
            .map(|(name, value)| (name.clone(), Value::String(value.clone())))
            .collect::<Map<String, Value>>(),
    );
    let manual = manual.with_variables(&variables);
    if event == EventName::WebhookReceived {
        manual.with_body(body)
    } else {
        manual
    }
}
