use std::collections::BTreeSet;

use super::checking::templates_of;
use super::evaluating::{every_step, placeholders_in};
use portal_feature::{Action, ApiError, Area, Right, Rights};

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

pub const READ_SECRETS: Right = Right::new(Area::Secrets, Action::Read);
pub const SECRETS: &str = "secrets.";

pub fn secret_fields(workflow: &Workflow) -> Vec<String> {
    let mut found = Vec::new();
    every_step(&workflow.steps, "steps", &mut |step, path| {
        for (field, template) in templates_of(step) {
            let named =
                placeholders_in(template).into_iter().any(|placeholder| {
                    placeholder.name.starts_with(SECRETS)
                        || placeholder.filters.iter().flatten().any(|call| {
                            call.names.iter().any(|(_, name)| name.starts_with(SECRETS))
                        })
                });
            if named {
                found.push(format!("{path}.{field}"));
            }
        }
    });
    found
}

pub fn secrets_allowed(workflow: &Workflow, rights: &Rights) -> Result<(), ApiError> {
    if rights.allows(READ_SECRETS) {
        return Ok(());
    }
    match secret_fields(workflow).into_iter().next() {
        Some(field) => Err(ApiError::Forbidden(format!(
            "{field} names a secret, which needs {READ_SECRETS}"
        ))),
        None => Ok(()),
    }
}
