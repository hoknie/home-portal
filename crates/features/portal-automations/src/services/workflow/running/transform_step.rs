use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{Map, Value};

use super::runner::WorkflowRunner;
use crate::services::workflow::checking::decode_transform;
use crate::services::workflow::evaluating::{
    Frame, Secrets, apply_chain, at_key, holds, order_of, render_text, render_value,
    rendered_arguments, text_of,
};
use crate::types::{
    Ending, Flow, Operation, RawOperation, RawStep, StepKind, StepLog, StepReport, TraceEntry,
    ValueType,
};

pub fn run_transform(input: &str, operations: &[Operation], frame: &mut Frame) -> StepReport {
    let mut value = match render_value(input, frame) {
        Ok(value) => value,
        Err(message) => return StepReport::failed(message),
    };
    let budget = TraceEntry::LONGEST_OUTPUT / (2 * operations.len().max(1));
    let mut snapshots = Vec::with_capacity(operations.len());
    let mut lines = Vec::with_capacity(operations.len());
    for (index, operation) in operations.iter().enumerate() {
        value = match apply_operation(value, operation, frame) {
            Ok(value) => value,
            Err((position, name, message)) => {
                return StepReport::failed(format!(
                    "operation {}{position} ({name}): {message}",
                    index + 1
                ));
            }
        };
        snapshots.push(snapshot(&value, budget));
        lines.push(format!(
            "after {} ({}): {}",
            index + 1,
            operation.name(),
            StepLog::shortened(&value.to_string(), StepLog::LONGEST)
        ));
    }
    let names: Vec<&str> = operations.iter().map(Operation::name).collect();
    StepReport {
        output: Some(Value::Array(snapshots).to_string()),
        ..StepReport::done(
            WorkflowRunner::step_result(&[("value", value)]),
            names.join(" → "),
        )
        .with_log(lines)
    }
}

pub const SAMPLE_INPUT: &str = "{{vars.input}}";

pub fn transform_sample(input: Value, operations: &[RawOperation]) -> Result<Value, String> {
    let raw = RawStep {
        kind: "transform".to_string(),
        input: Some(SAMPLE_INPUT.to_string()),
        operations: Some(operations.to_vec()),
        ..RawStep::default()
    };
    let mut errors = Vec::new();
    let Some(StepKind::Transform { operations, .. }) = decode_transform(&raw, "", &mut errors)
    else {
        let messages: Vec<String> = errors
            .iter()
            .map(|error| format!("{}: {}", error.field.trim_start_matches('.'), error.message))
            .collect();
        return Err(messages.join("; "));
    };
    let mut frame = Frame::new(
        Arc::new(Vec::new()),
        BTreeMap::new(),
        Arc::new(Secrets::new(Arc::new(|_| None))),
    );
    frame.vars.insert("input".to_string(), input);
    let report = run_transform(SAMPLE_INPUT, &operations, &mut frame);
    match report.flow {
        Flow::End(Ending::Failed(message)) => Err(message),
        _ => Ok(report.result.get("value").cloned().unwrap_or(Value::Null)),
    }
}

pub type Failure = (String, String, String);

fn failure(operation: &Operation, message: String) -> Failure {
    (String::new(), operation.name().to_string(), message)
}

pub fn apply_operation(
    value: Value,
    operation: &Operation,
    frame: &mut Frame,
) -> Result<Value, Failure> {
    let Operation::Filter(call) = operation else {
        let items = match value {
            Value::Array(items) => items,
            Value::Null => return Ok(Value::Null),
            other => {
                let message = format!(
                    "{} takes a list and got {}",
                    operation.name(),
                    ValueType::of(&other).described()
                );
                return Err(failure(operation, message));
            }
        };
        if let Operation::Each(chain) = operation {
            return each(items, chain, frame);
        }
        return list_operation(items, operation, frame)
            .map_err(|message| failure(operation, message));
    };
    let call = rendered_arguments(call, &|template| render_value(template, frame))
        .map_err(|message| failure(operation, message))?;
    apply_chain(value, std::slice::from_ref(&call), &|name| {
        frame.lookup(name)
    })
    .map_err(|message| failure(operation, message))
}

fn each(items: Vec<Value>, chain: &[Operation], frame: &mut Frame) -> Result<Value, Failure> {
    let mut results = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        let previous = frame.transform_item.replace((item.clone(), index));
        let mut value = item;
        for (position, operation) in chain.iter().enumerate() {
            match apply_operation(value, operation, frame) {
                Ok(next) => value = next,
                Err((inner, name, message)) => {
                    frame.transform_item = previous;
                    return Err((format!(".{}{inner}", position + 1), name, message));
                }
            }
        }
        frame.transform_item = previous;
        results.push(value);
    }
    Ok(Value::Array(results))
}

fn list_operation(
    items: Vec<Value>,
    operation: &Operation,
    frame: &mut Frame,
) -> Result<Value, String> {
    match operation {
        Operation::Where(condition) => {
            let mut kept = Vec::new();
            for (index, item) in items.into_iter().enumerate() {
                if with_item(frame, &item, index, |frame| holds(condition, frame))? {
                    kept.push(item);
                }
            }
            Ok(Value::Array(kept))
        }
        Operation::Map(to) => items
            .iter()
            .enumerate()
            .map(|(index, item)| with_item(frame, item, index, |frame| render_value(to, frame)))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Operation::SortBy { key, descending } => {
            let key = &key_of(key, frame)?;
            let mut items = items;
            items.sort_by(|left, right| {
                let order = order_of(&at_key(left.clone(), key), &at_key(right.clone(), key));
                if *descending { order.reverse() } else { order }
            });
            Ok(Value::Array(items))
        }
        Operation::GroupBy(key) => {
            let key = &key_of(key, frame)?;
            let mut groups: Map<String, Value> = Map::new();
            for item in items {
                let name = text_of(&at_key(item.clone(), key));
                if let Value::Array(members) = groups
                    .entry(name)
                    .or_insert_with(|| Value::Array(Vec::new()))
                {
                    members.push(item);
                }
            }
            Ok(Value::Object(groups))
        }
        Operation::CountBy(key) => {
            let key = &key_of(key, frame)?;
            let mut counts: Map<String, Value> = Map::new();
            for item in items {
                let name = text_of(&at_key(item, key));
                let count = counts.get(&name).and_then(Value::as_u64).unwrap_or(0);
                counts.insert(name, Value::from(count + 1));
            }
            Ok(Value::Object(counts))
        }
        Operation::Filter(_) | Operation::Each(_) => Ok(Value::Array(items)),
    }
}

fn key_of(key: &str, frame: &Frame) -> Result<String, String> {
    if key.contains(crate::helpers::OPEN) {
        render_text(key, frame)
    } else {
        Ok(key.to_string())
    }
}

fn with_item<T>(
    frame: &mut Frame,
    item: &Value,
    index: usize,
    run: impl FnOnce(&Frame) -> Result<T, String>,
) -> Result<T, String> {
    let previous = frame.transform_item.replace((item.clone(), index));
    let result = run(frame);
    frame.transform_item = previous;
    result
}

fn snapshot(value: &Value, budget: usize) -> Value {
    let text = value.to_string();
    if text.len() <= budget {
        return value.clone();
    }
    let mut end = budget / 2;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Value::String(format!("{}…", &text[..end]))
}
