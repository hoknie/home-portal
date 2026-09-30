use portal_feature::{EventName, FieldError, PortalEvent};

use crate::helpers::{body_path, placeholders_of};
use crate::types::{Automation, Catalogue, Webhook, WebhookAction};

pub fn webhook_placeholder_errors(
    automation: &Automation,
    webhooks: &[Webhook],
) -> Vec<FieldError> {
    if automation.trigger.event != EventName::WebhookReceived {
        return Vec::new();
    }
    let chosen = &automation.trigger.filters.webhooks;
    let scope: Vec<&Webhook> = webhooks
        .iter()
        .filter(|webhook| {
            if chosen.is_empty() {
                webhook.action == WebhookAction::Event
            } else {
                chosen.contains(&webhook.id)
            }
        })
        .collect();
    let fixed = Catalogue::fields_of(EventName::WebhookReceived);
    let declared_by_all = |variable: &str| {
        !scope.is_empty()
            && scope
                .iter()
                .all(|webhook| webhook.variables.iter().any(|name| name == variable))
    };
    let run = Some(automation.run.clone());
    let mut errors = Vec::new();
    for (field, template) in Automation::templates_of(&run, automation.workflow.as_ref()) {
        let missing = placeholders_of(template).into_iter().find(|name| {
            !fixed.contains(name)
                && body_path(name).is_none()
                && name
                    .strip_prefix(PortalEvent::VARIABLE_PREFIX)
                    .is_some_and(|variable| !declared_by_all(variable))
        });
        if let Some(name) = missing {
            errors.push(FieldError::new(
                field,
                format!("names {{{{{name}}}}}, which not every chosen webhook declares"),
            ));
        }
    }
    errors
}
