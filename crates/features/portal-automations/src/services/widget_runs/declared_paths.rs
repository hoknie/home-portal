use serde_json::Value;

use crate::helpers::{CLOSE, OPEN};
use crate::services::workflow::every_step;
use crate::types::{StepKind, Workflow};

pub const MOST_PATHS: usize = 200;
pub const DEEPEST_PATH: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredPath {
    pub path: String,
    pub description: Option<String>,
    pub kind: &'static str,
}

pub fn declared_paths(workflow: &Workflow) -> Vec<DeclaredPath> {
    let mut paths = Vec::new();
    for output in &workflow.outputs {
        let root = format!("data.{}", output.name);
        let sample = sample_behind(workflow, &output.value);
        paths.push(DeclaredPath {
            path: root.clone(),
            description: output.description.clone(),
            kind: sample.as_ref().map_or("value", kind_of),
        });
        if let Some(sample) = &sample {
            inside(sample, &root, 1, &mut paths);
        }
        if paths.len() >= MOST_PATHS {
            paths.truncate(MOST_PATHS);
            break;
        }
    }
    paths
}

fn kind_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "value",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "text",
        Value::Array(_) => "list",
        Value::Object(_) => "object",
    }
}

fn inside(value: &Value, path: &str, depth: usize, paths: &mut Vec<DeclaredPath>) {
    if depth > DEEPEST_PATH || paths.len() >= MOST_PATHS {
        return;
    }
    let children: Vec<(String, &Value)> = match value {
        Value::Object(fields) => fields
            .iter()
            .map(|(key, child)| (key.clone(), child))
            .collect(),
        Value::Array(items) => items
            .first()
            .map(|first| ("0".to_string(), first))
            .into_iter()
            .collect(),
        _ => Vec::new(),
    };
    for (key, child) in children {
        let here = format!("{path}.{key}");
        paths.push(DeclaredPath {
            path: here.clone(),
            description: None,
            kind: kind_of(child),
        });
        inside(child, &here, depth + 1, paths);
    }
}

fn sample_behind(workflow: &Workflow, template: &str) -> Option<Value> {
    let name = template
        .trim()
        .strip_prefix(OPEN)?
        .strip_suffix(CLOSE)?
        .trim();
    if name.contains(OPEN) || name.contains('|') {
        return None;
    }
    let mut parts = name.split('.');
    if parts.next() != Some("steps") {
        return None;
    }
    let id = parts.next()?;
    if parts.next() != Some("json") {
        return None;
    }
    let mut found = None;
    every_step(&workflow.steps, "steps", &mut |step, _| {
        if step.id == id
            && let StepKind::Http(http) = &step.kind
        {
            found = http.response_sample.clone();
        }
    });
    let mut value: Value = serde_json::from_str(&found?).ok()?;
    for key in parts {
        value = match value {
            Value::Object(mut fields) => fields.remove(key)?,
            Value::Array(mut items) => {
                let index: usize = key.parse().ok()?;
                (index < items.len()).then(|| items.swap_remove(index))?
            }
            _ => return None,
        };
    }
    Some(value)
}
