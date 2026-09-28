use portal_feature::FieldError;

use super::condition_decoding::decode_condition;
use super::decoding::decode_steps;
use super::names::{NAME_RULE, between_rule, number_setting, valid_name};
use crate::types::{LoopMode, OUTCOMES, RawStep, SetValue, Step, StepKind, Workflow};

pub type Place<'a> = (&'a str, usize);

pub fn decode_if(
    raw: &RawStep,
    (path, depth): Place,
    errors: &mut Vec<FieldError>,
) -> Option<StepKind> {
    let condition = match &raw.condition {
        Some(condition) => decode_condition(condition, &format!("{path}.condition"), 0, errors),
        None => {
            errors.push(FieldError::new(format!("{path}.condition"), "is required"));
            None
        }
    };
    let then = required_steps(raw.then.as_deref(), &format!("{path}.then"), depth, errors);
    let otherwise = decode_steps(
        raw.otherwise.as_deref().unwrap_or_default(),
        &format!("{path}.else"),
        depth + 1,
        errors,
    );
    Some(StepKind::If {
        condition: condition?,
        then: then?,
        otherwise,
    })
}

pub fn decode_loop(
    raw: &RawStep,
    (path, depth): Place,
    errors: &mut Vec<FieldError>,
) -> Option<StepKind> {
    let chosen = [
        raw.repeat.is_some(),
        raw.for_each.is_some(),
        raw.while_condition.is_some(),
    ];
    let mode = match chosen.iter().filter(|given| **given).count() {
        1 => loop_mode(raw, path, errors),
        _ => {
            errors.push(FieldError::new(
                path,
                "needs exactly one of repeat, for_each and while",
            ));
            None
        }
    };
    let most = i64::from(Workflow::MOST_ITERATIONS);
    let max_iterations = number_setting(
        raw.max_iterations.as_ref(),
        (Some(most as u64), 1, most),
        format!("{path}.max_iterations"),
        errors,
    );
    let body = match &raw.body {
        Some(value) => match value.clone().try_into::<Vec<RawStep>>() {
            Ok(steps) => required_steps(Some(&steps), &format!("{path}.body"), depth, errors),
            Err(_) => {
                errors.push(FieldError::new(
                    format!("{path}.body"),
                    "must be a list of steps",
                ));
                None
            }
        },
        None => required_steps(None, &format!("{path}.body"), depth, errors),
    };
    Some(StepKind::Loop {
        mode: mode?,
        max_iterations: max_iterations?,
        body: body?,
    })
}

fn loop_mode(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<LoopMode> {
    if raw.repeat.is_some() {
        return number_setting(
            raw.repeat.as_ref(),
            (None, 1, i64::from(Workflow::MOST_ITERATIONS)),
            format!("{path}.repeat"),
            errors,
        )
        .map(LoopMode::Repeat);
    }
    if let Some(list) = &raw.for_each {
        return Some(LoopMode::ForEach(list.clone()));
    }
    let condition = raw.while_condition.as_ref()?;
    decode_condition(condition, &format!("{path}.while"), 0, errors).map(LoopMode::While)
}

pub fn decode_parallel(
    raw: &RawStep,
    (path, depth): Place,
    errors: &mut Vec<FieldError>,
) -> Option<StepKind> {
    let branches = raw.branches.as_deref().unwrap_or_default();
    if !(Workflow::FEWEST_BRANCHES..=Workflow::MOST_BRANCHES).contains(&branches.len()) {
        errors.push(FieldError::new(
            format!("{path}.branches"),
            format!(
                "must hold {} to {} branches",
                Workflow::FEWEST_BRANCHES,
                Workflow::MOST_BRANCHES
            ),
        ));
        return None;
    }
    let decoded: Vec<Option<Vec<Step>>> = branches
        .iter()
        .enumerate()
        .map(|(index, branch)| {
            required_steps(
                Some(branch),
                &format!("{path}.branches[{index}]"),
                depth,
                errors,
            )
        })
        .collect();
    Some(StepKind::Parallel {
        branches: decoded.into_iter().collect::<Option<Vec<_>>>()?,
    })
}

pub fn decode_call(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    let Some(workflow) = raw.workflow.clone().filter(|id| !id.trim().is_empty()) else {
        errors.push(FieldError::new(format!("{path}.workflow"), "is required"));
        return None;
    };
    Some(StepKind::Call {
        workflow,
        inputs: raw.inputs.clone().unwrap_or_default().into_iter().collect(),
    })
}

pub fn decode_stop(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    match raw.outcome.as_deref() {
        Some(outcome) if OUTCOMES.contains(&outcome) => Some(StepKind::Stop {
            succeeded: outcome == "succeeded",
            reason: raw.reason.clone(),
        }),
        _ => {
            errors.push(FieldError::new(
                format!("{path}.outcome"),
                format!("must be one of {}", OUTCOMES.join(", ")),
            ));
            None
        }
    }
}

pub const SET_RULE: &str = "needs exactly one of value, json, list and object";

pub fn decode_set(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    let variable = raw.variable.clone().unwrap_or_default();
    if !valid_name(&variable) {
        errors.push(FieldError::new(format!("{path}.variable"), NAME_RULE));
    }
    let given = [
        raw.value.is_some(),
        raw.json.is_some(),
        raw.list.is_some(),
        raw.object.is_some(),
    ];
    let value = match (given.iter().filter(|given| **given).count(), raw) {
        (
            1,
            RawStep {
                value: Some(text), ..
            },
        ) => Some(SetValue::Text(text.clone())),
        (
            1,
            RawStep {
                json: Some(json), ..
            },
        ) => Some(SetValue::Json(json.clone())),
        (
            1,
            RawStep {
                list: Some(list), ..
            },
        ) => Some(SetValue::List(list.clone())),
        (
            1,
            RawStep {
                object: Some(object),
                ..
            },
        ) => Some(SetValue::Object(
            object
                .iter()
                .map(|(key, template)| (key.clone(), template.clone()))
                .collect(),
        )),
        _ => {
            errors.push(FieldError::new(path, SET_RULE));
            None
        }
    };
    Some(StepKind::Set {
        variable,
        value: value?,
    })
}

pub fn decode_wait(raw: &RawStep, path: &str, errors: &mut Vec<FieldError>) -> Option<StepKind> {
    let longest = Workflow::LONGEST_WAIT as i64;
    if raw.seconds.is_none() {
        errors.push(FieldError::new(
            format!("{path}.seconds"),
            between_rule(1, longest),
        ));
        return None;
    }
    number_setting(
        raw.seconds.as_ref(),
        (None, 1, longest),
        format!("{path}.seconds"),
        errors,
    )
    .map(|seconds| StepKind::Wait { seconds })
}

fn required_steps(
    raw: Option<&[RawStep]>,
    path: &str,
    depth: usize,
    errors: &mut Vec<FieldError>,
) -> Option<Vec<Step>> {
    match raw {
        Some(steps) if !steps.is_empty() => {
            let before = errors.len();
            let decoded = decode_steps(steps, path, depth + 1, errors);
            (errors.len() == before || decoded.len() == steps.len()).then_some(decoded)
        }
        _ => {
            errors.push(FieldError::new(path, "must hold at least one step"));
            None
        }
    }
}
