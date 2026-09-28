use std::collections::HashSet;

use portal_feature::FieldError;

use super::calls::{automation_errors, call_errors};
use super::decoding::decode_workflow;
use super::scope::Scope;
use crate::types::{AutomationsSection, Workflow};

pub fn workflow_errors(section: &AutomationsSection) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    let mut decoded = Vec::new();
    for (index, raw) in section.workflows.iter().enumerate() {
        let prefix = format!("{}[{index}].", Workflow::SECTION);
        match decode_workflow(raw) {
            Ok(workflow) => {
                errors.extend(
                    Scope::errors(&workflow, section.service_ids())
                        .into_iter()
                        .map(|error| error.prefixed(&prefix)),
                );
                decoded.push((index, workflow));
            }
            Err(found) => errors.extend(found.into_iter().map(|error| error.prefixed(&prefix))),
        }
        if !raw.id.is_empty() && !seen.insert(raw.id.as_str()) {
            errors.push(FieldError::new(format!("{prefix}id"), Workflow::TAKEN_ID));
        }
    }
    errors.extend(call_errors(&decoded));
    errors.extend(automation_errors(&decoded, &section.automations));
    errors
}

pub fn decoded_workflows(section: &AutomationsSection) -> Vec<Workflow> {
    section
        .workflows
        .iter()
        .filter_map(|raw| decode_workflow(raw).ok())
        .collect()
}

pub fn entry_errors(section: &AutomationsSection, index: usize) -> Vec<FieldError> {
    let prefix = format!("{}[{index}].", Workflow::SECTION);
    workflow_errors(section)
        .into_iter()
        .filter_map(|error| {
            error
                .field
                .strip_prefix(&prefix)
                .map(|field| FieldError::new(field, error.message.clone()))
        })
        .collect()
}
