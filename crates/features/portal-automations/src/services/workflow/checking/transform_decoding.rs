use portal_feature::FieldError;

use super::condition_decoding::decode_condition;
use super::filter_checks::arguments_problem;
use crate::types::{
    ArgumentType, DEEPEST_EACH, FilterCall, MOST_OPERATIONS, OPERATIONS, Operation, RawOperation,
    RawStep, StepKind, filter_named,
};

pub fn decode_transform(
    raw: &RawStep,
    path: &str,
    errors: &mut Vec<FieldError>,
) -> Option<StepKind> {
    let before = errors.len();
    let input = raw.input.clone().filter(|input| !input.trim().is_empty());
    if input.is_none() {
        errors.push(FieldError::new(format!("{path}.input"), "is required"));
    }
    let raw_operations = raw.operations.clone().unwrap_or_default();
    if raw_operations.is_empty() || raw_operations.len() > MOST_OPERATIONS {
        errors.push(FieldError::new(
            format!("{path}.operations"),
            format!("must hold 1 to {MOST_OPERATIONS} operations"),
        ));
    }
    let operations = decode_chain(&raw_operations, &format!("{path}.operations"), 0, errors);
    (errors.len() == before).then(|| StepKind::Transform {
        input: input.unwrap_or_default(),
        operations,
    })
}

fn decode_chain(
    raw: &[RawOperation],
    path: &str,
    depth: usize,
    errors: &mut Vec<FieldError>,
) -> Vec<Operation> {
    raw.iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            decode_operation(operation, (&format!("{path}[{index}]"), depth), errors)
        })
        .collect()
}

fn decode_each(
    raw: &RawOperation,
    (path, depth): (&str, usize),
    errors: &mut Vec<FieldError>,
) -> Option<Operation> {
    let nested = raw.operations.clone().unwrap_or_default();
    if depth >= DEEPEST_EACH {
        errors.push(FieldError::new(
            path,
            format!("each nests at most {DEEPEST_EACH} deep"),
        ));
        return None;
    }
    if nested.is_empty() || nested.len() > MOST_OPERATIONS {
        errors.push(FieldError::new(
            format!("{path}.operations"),
            format!("must hold 1 to {MOST_OPERATIONS} operations"),
        ));
        return None;
    }
    let before = errors.len();
    let chain = decode_chain(&nested, &format!("{path}.operations"), depth + 1, errors);
    (errors.len() == before).then_some(Operation::Each(chain))
}

fn decode_operation(
    raw: &RawOperation,
    (path, depth): (&str, usize),
    errors: &mut Vec<FieldError>,
) -> Option<Operation> {
    if raw.op == "each" {
        return decode_each(raw, (path, depth), errors);
    }
    let key = || {
        raw.key
            .clone()
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| FieldError::new(format!("{path}.key"), "is required"))
    };
    let decoded = match raw.op.as_str() {
        "filter" => match &raw.condition {
            Some(condition) => {
                return decode_condition(condition, &format!("{path}.where"), 0, errors)
                    .map(Operation::Where);
            }
            None => Err(FieldError::new(format!("{path}.where"), "is required")),
        },
        "map" => raw
            .to
            .clone()
            .filter(|to| !to.trim().is_empty())
            .map(Operation::Map)
            .ok_or_else(|| FieldError::new(format!("{path}.to"), "is required")),
        "sort_by" => match raw.order.as_deref().unwrap_or("asc") {
            order if ArgumentType::ORDERS.contains(&order) => key().map(|key| Operation::SortBy {
                key,
                descending: order == "desc",
            }),
            _ => Err(FieldError::new(
                format!("{path}.order"),
                format!("must be one of {}", ArgumentType::ORDERS.join(", ")),
            )),
        },
        "group_by" => key().map(Operation::GroupBy),
        "count_by" => key().map(Operation::CountBy),
        name => match filter_named(name) {
            Some(description) => {
                let arguments = raw.args.clone().unwrap_or_default();
                match arguments_problem(description, &arguments) {
                    Some(problem) => Err(FieldError::new(format!("{path}.args"), problem)),
                    None => Ok(Operation::Filter(FilterCall {
                        name: name.to_string(),
                        arguments,
                    })),
                }
            }
            None => {
                let names: Vec<&str> = OPERATIONS.iter().map(|operation| operation.name).collect();
                Err(FieldError::new(
                    format!("{path}.op"),
                    format!("must be {} or a filter such as join", names.join(", ")),
                ))
            }
        },
    };
    decoded.map_err(|error| errors.push(error)).ok()
}
