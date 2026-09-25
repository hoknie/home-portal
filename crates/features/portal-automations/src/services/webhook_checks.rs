use portal_feature::{EventName, FieldError, PortalEvent};

use crate::helpers::placeholders_of;
use crate::types::{Automation, Catalogue, Webhook};

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
        .filter(|webhook| chosen.is_empty() || chosen.contains(&webhook.id))
        .collect();
    let fixed = Catalogue::fields_of(EventName::WebhookReceived);
    let mut errors = Vec::new();
    let declared_by_all = |variable: &str| {
        !scope.is_empty()
            && scope
                .iter()
                .all(|webhook| webhook.variables.iter().any(|name| name == variable))
    };
    for (index, argument) in automation.run.args.iter().enumerate() {
        let missing = placeholders_of(argument).into_iter().find(|name| {
            !fixed.contains(name)
                && name
                    .strip_prefix(PortalEvent::VARIABLE_PREFIX)
                    .is_some_and(|variable| !declared_by_all(variable))
        });
        if let Some(name) = missing {
            errors.push(FieldError::new(
                format!("run.args[{index}]"),
                format!("names {{{{{name}}}}}, which not every chosen webhook declares"),
            ));
        }
    }
    errors
}
