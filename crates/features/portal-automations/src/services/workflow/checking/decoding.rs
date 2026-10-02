use std::collections::HashSet;

use portal_feature::FieldError;

use super::action_decoding::{
    decode_automation, decode_http, decode_log, decode_notify, decode_script, decode_service,
};
use super::flow_decoding::{
    check_loop_exits, decode_call, decode_if, decode_loop, decode_parallel, decode_set,
    decode_stop, decode_wait,
};
use super::inputs_decoding::{decode_inputs, decode_outputs};
use super::names::{NAME_RULE, between_rule, valid_name, within};
use super::transform_decoding::decode_transform;
use crate::helpers::{check_tags, version_of};
use crate::services::workflow::evaluating::every_step;
use crate::types::{KINDS, RawStep, RawWorkflow, Step, StepKind, Workflow, kind_named};

pub fn decode_workflow(raw: &RawWorkflow) -> Result<Workflow, Vec<FieldError>> {
    let mut errors = Vec::new();
    if Workflow::RESERVED_IDS.contains(&raw.id.as_str()) {
        errors.push(FieldError::new(
            "id",
            format!(
                "is reserved; {} cannot be used",
                Workflow::RESERVED_IDS.join(", ")
            ),
        ));
    } else if !valid_workflow_id(&raw.id) {
        errors.push(FieldError::new(
            "id",
            format!(
                "must be 1 to {} lowercase letters, digits and -",
                Workflow::LONGEST_ID
            ),
        ));
    }
    if raw.title.trim().is_empty() {
        errors.push(FieldError::new("title", "must not be empty"));
    }
    errors.extend(check_tags(&raw.tags));
    let timeout = raw
        .timeout_seconds
        .unwrap_or(Workflow::DEFAULT_TIMEOUT as i64);
    if !within(timeout, 1, Workflow::LONGEST_TIMEOUT as i64) {
        errors.push(FieldError::new(
            "timeout_seconds",
            between_rule(1, Workflow::LONGEST_TIMEOUT as i64),
        ));
    }
    let inputs = decode_inputs(&raw.inputs, &mut errors);
    let outputs = decode_outputs(&raw.outputs, &mut errors);
    if raw.steps.is_empty() {
        errors.push(FieldError::new("steps", "must hold at least one step"));
    }
    let steps = decode_steps(&raw.steps, "steps", 0, &mut errors);
    check_loop_exits(&steps, "steps", false, &mut errors);
    let mut seen = HashSet::new();
    every_step(&steps, "steps", &mut |step, path| {
        if !seen.insert(step.id.clone()) {
            errors.push(FieldError::new(
                format!("{path}.id"),
                "is used by another step of this workflow",
            ));
        }
    });
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Workflow {
        id: raw.id.clone(),
        title: raw.title.clone(),
        enabled: raw.enabled.unwrap_or(true),
        description: raw
            .description
            .clone()
            .filter(|text| !text.trim().is_empty()),
        tags: raw.tags.clone(),
        timeout_seconds: timeout as u64,
        inputs,
        outputs,
        steps,
        version: version_of(&serde_json::to_string(&raw.steps).unwrap_or_default()),
    })
}

pub fn valid_workflow_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= Workflow::LONGEST_ID
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn decode_steps(
    raw: &[RawStep],
    path: &str,
    depth: usize,
    errors: &mut Vec<FieldError>,
) -> Vec<Step> {
    raw.iter()
        .enumerate()
        .filter_map(|(index, step)| decode_step(step, &format!("{path}[{index}]"), depth, errors))
        .collect()
}

fn decode_step(
    raw: &RawStep,
    path: &str,
    depth: usize,
    errors: &mut Vec<FieldError>,
) -> Option<Step> {
    if !valid_name(&raw.id) {
        errors.push(FieldError::new(format!("{path}.id"), NAME_RULE));
    }
    let kind = raw.kind.as_str();
    let Some(description) = kind_named(kind) else {
        let names: Vec<&str> = KINDS.iter().map(|kind| kind.name).collect();
        errors.push(FieldError::new(
            format!("{path}.kind"),
            format!("must be one of {}", names.join(", ")),
        ));
        return None;
    };
    let container = matches!(description.name, "if" | "loop" | "parallel");
    if container && depth >= Workflow::DEEPEST_NESTING {
        errors.push(FieldError::new(
            path,
            format!(
                "nests too deep; if, loop and parallel nest at most {} deep",
                Workflow::DEEPEST_NESTING
            ),
        ));
        return None;
    }
    let kind = match description.name {
        "if" => decode_if(raw, (path, depth), errors),
        "loop" => decode_loop(raw, (path, depth), errors),
        "parallel" => decode_parallel(raw, (path, depth), errors),
        "workflow" => decode_call(raw, path, errors),
        "stop" => decode_stop(raw, path, errors),
        "set" => decode_set(raw, path, errors),
        "wait" => decode_wait(raw, path, errors),
        "transform" => decode_transform(raw, path, errors),
        "http" => decode_http(raw, path, errors),
        "script" => decode_script(raw, path, errors),
        "notify" => decode_notify(raw, path, errors),
        "log" => decode_log(raw, path, errors),
        "nothing" => Some(StepKind::Nothing),
        "break" => Some(StepKind::Break),
        "continue" => Some(StepKind::Continue),
        "automation" => decode_automation(raw, path, errors),
        "probe" => decode_service(raw, path, errors).map(|service| StepKind::Probe { service }),
        _ => decode_service(raw, path, errors).map(|service| StepKind::Status { service }),
    }?;
    Some(Step {
        id: raw.id.clone(),
        label: raw
            .label
            .clone()
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_else(|| raw.id.clone()),
        kind,
    })
}
