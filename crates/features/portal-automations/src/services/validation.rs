use std::collections::HashSet;

use portal_feature::FieldError;
use toml_edit::DocumentMut;

use super::{decoded_workflows, webhook_placeholder_errors, workflow_errors};
use crate::types::{
    Automation, AutomationsSection, InputValue, Webhook, WebhookAction, Workflow, WorkflowCall,
};

pub fn validate_automations(document: &DocumentMut) -> Vec<FieldError> {
    let section = match AutomationsSection::read(document) {
        Ok(section) => section,
        Err(message) => return vec![FieldError::new(AutomationsSection::SECTION, message)],
    };
    let mut errors = Vec::new();
    if let Err(message) = section.automation_settings.zone() {
        errors.push(FieldError::new(
            format!("{}.timezone", AutomationsSection::SETTINGS),
            message,
        ));
    }
    let webhooks = webhooks_of(&section, &mut errors);
    errors.extend(workflow_errors(&section));
    let workflows = decoded_workflows(&section);
    for (index, raw) in section.webhooks.iter().enumerate() {
        if let Ok(Webhook {
            action: WebhookAction::Workflow(call),
            ..
        }) = Webhook::decode(raw)
        {
            let prefix = format!("{}[{index}].", AutomationsSection::WEBHOOKS);
            errors.extend(
                call_problems(&call, &workflows)
                    .into_iter()
                    .map(|error| error.prefixed(&prefix)),
            );
        }
    }
    let mut seen = HashSet::new();
    for (index, raw) in section.automations.iter().enumerate() {
        let prefix = format!("{}[{index}].", AutomationsSection::SECTION);
        match Automation::decode(raw) {
            Err(found) => errors.extend(found.into_iter().map(|error| error.prefixed(&prefix))),
            Ok(automation) => {
                errors.extend(
                    webhook_placeholder_errors(&automation, &webhooks)
                        .into_iter()
                        .map(|error| error.prefixed(&prefix)),
                );
                if let Some(call) = &automation.workflow {
                    errors.extend(
                        call_problems(call, &workflows)
                            .into_iter()
                            .map(|error| error.prefixed(&prefix)),
                    );
                }
            }
        }
        if !raw.id.is_empty() && !seen.insert(raw.id.as_str()) {
            errors.push(FieldError::new(
                format!("{prefix}id"),
                "is used by another automation",
            ));
        }
    }
    errors
}

pub fn automation_problems(document: &DocumentMut, automation: &Automation) -> Vec<FieldError> {
    let Ok(section) = AutomationsSection::read(document) else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    let webhooks = webhooks_of(&section, &mut errors);
    let mut problems = webhook_placeholder_errors(automation, &webhooks);
    if let Some(call) = &automation.workflow {
        problems.extend(call_problems(call, &decoded_workflows(&section)));
    }
    problems
}

pub fn webhook_problems(document: &DocumentMut, webhook: &Webhook) -> Vec<FieldError> {
    let WebhookAction::Workflow(call) = &webhook.action else {
        return Vec::new();
    };
    AutomationsSection::read(document)
        .map(|section| call_problems(call, &decoded_workflows(&section)))
        .unwrap_or_default()
}

pub fn decoded(document: &DocumentMut) -> Vec<Automation> {
    AutomationsSection::read(document)
        .map(|section| {
            section
                .automations
                .iter()
                .filter_map(|raw| Automation::decode(raw).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn webhooks_of(section: &AutomationsSection, errors: &mut Vec<FieldError>) -> Vec<Webhook> {
    let mut seen = HashSet::new();
    let mut webhooks = Vec::new();
    for (index, raw) in section.webhooks.iter().enumerate() {
        let prefix = format!("{}[{index}].", AutomationsSection::WEBHOOKS);
        match Webhook::decode(raw) {
            Ok(webhook) => webhooks.push(webhook),
            Err(found) => errors.extend(found.into_iter().map(|error| error.prefixed(&prefix))),
        }
        if !raw.id.is_empty() && !seen.insert(raw.id.as_str()) {
            errors.push(FieldError::new(
                format!("{prefix}id"),
                "is used by another webhook",
            ));
        }
    }
    webhooks
}

pub fn decoded_webhooks(document: &DocumentMut) -> Vec<Webhook> {
    AutomationsSection::read(document)
        .map(|section| {
            section
                .webhooks
                .iter()
                .filter_map(|raw| Webhook::decode(raw).ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn call_problems(call: &WorkflowCall, workflows: &[Workflow]) -> Vec<FieldError> {
    let Some(workflow) = workflows.iter().find(|workflow| workflow.id == call.id) else {
        return vec![FieldError::new(
            "workflow",
            format!("names {:?}, which is not a workflow", call.id),
        )];
    };
    call.inputs
        .iter()
        .filter_map(|(name, value)| match (workflow.input(name), value) {
            (None, _) => Some(FieldError::new(
                format!("inputs.{name}"),
                format!("is not an input of {}", workflow.id),
            )),
            (Some(input), InputValue::Literal(literal)) if !input.input_type.fits(literal) => {
                Some(FieldError::new(
                    format!("inputs.{name}"),
                    format!("must be {}", input.input_type.described()),
                ))
            }
            _ => None,
        })
        .collect()
}
