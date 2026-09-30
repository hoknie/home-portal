use std::collections::BTreeSet;

use super::checking::templates_of;
use super::evaluating::{every_step, placeholders_in};
use crate::types::Workflow;

pub const EVENT_WEBHOOK: &str = "event.webhook.";

pub fn webhook_variables_read(workflow: &Workflow) -> Vec<String> {
    let mut found = BTreeSet::new();
    every_step(&workflow.steps, "steps", &mut |step, _| {
        for (_, template) in templates_of(step) {
            for placeholder in placeholders_in(template) {
                let variable = placeholder
                    .name
                    .strip_prefix(EVENT_WEBHOOK)
                    .and_then(|rest| rest.split('.').next())
                    .filter(|name| !["id", "title", "body"].contains(name));
                if let Some(name) = variable {
                    found.insert(name.to_string());
                }
            }
        }
    });
    found.into_iter().collect()
}
