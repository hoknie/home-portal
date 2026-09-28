use crate::types::{Condition, LoopMode, Operation, SetValue, Step, StepKind};

pub fn templates_of(step: &Step) -> Vec<(String, &str)> {
    let mut found: Vec<(String, &str)> = Vec::new();
    match &step.kind {
        StepKind::If { condition, .. } => condition_templates(condition, "condition", &mut found),
        StepKind::Loop { mode, .. } => match mode {
            LoopMode::Repeat(_) => {}
            LoopMode::ForEach(list) => found.push(("for_each".to_string(), list)),
            LoopMode::While(condition) => condition_templates(condition, "while", &mut found),
        },
        StepKind::Parallel { .. } | StepKind::Wait { .. } | StepKind::Nothing => {}
        StepKind::Automation { fields, .. } => {
            for (name, value) in fields {
                found.push((format!("fields.{name}"), value));
            }
        }
        StepKind::Call { inputs, .. } => {
            for (name, value) in inputs {
                found.push((format!("inputs.{name}"), value));
            }
        }
        StepKind::Stop { reason, .. } => found.extend(
            reason
                .as_deref()
                .map(|reason| ("reason".to_string(), reason)),
        ),
        StepKind::Set { value, .. } => match value {
            SetValue::Text(text) => found.push(("value".to_string(), text)),
            SetValue::Json(json) => found.push(("json".to_string(), json)),
            SetValue::List(list) => {
                for (index, item) in list.iter().enumerate() {
                    found.push((format!("list[{index}]"), item));
                }
            }
            SetValue::Object(object) => {
                for (key, item) in object {
                    found.push((format!("object.{key}"), item));
                }
            }
        },
        StepKind::Transform { input, operations } => {
            found.push(("input".to_string(), input));
            operation_templates(operations, "operations", &mut found);
        }
        StepKind::Http(http) => {
            found.push(("url".to_string(), &http.url));
            for (name, value) in &http.headers {
                found.push((format!("headers.{name}"), value));
            }
            found.extend(http.body.as_deref().map(|body| ("body".to_string(), body)));
        }
        StepKind::Script { run, env, stdin } => {
            for (index, argument) in run.args.iter().enumerate() {
                found.push((format!("args[{index}]"), argument));
            }
            for (name, value) in env {
                found.push((format!("env.{name}"), value));
            }
            found.extend(stdin.as_deref().map(|stdin| ("stdin".to_string(), stdin)));
        }
        StepKind::Notify { title, text, .. } => {
            found.push(("text".to_string(), text));
            found.extend(title.as_deref().map(|title| ("title".to_string(), title)));
        }
        StepKind::Probe { service } | StepKind::Status { service } => {
            found.push(("service".to_string(), service));
        }
    }
    found
}

fn condition_templates<'a>(
    condition: &'a Condition,
    path: &str,
    found: &mut Vec<(String, &'a str)>,
) {
    match condition {
        Condition::Compare { left, right, .. } => {
            found.push((format!("{path}.left"), left));
            found.extend(
                right
                    .as_deref()
                    .map(|right| (format!("{path}.right"), right)),
            );
        }
        Condition::All(inner) => {
            for (index, condition) in inner.iter().enumerate() {
                condition_templates(condition, &format!("{path}.all[{index}]"), found);
            }
        }
        Condition::Any(inner) => {
            for (index, condition) in inner.iter().enumerate() {
                condition_templates(condition, &format!("{path}.any[{index}]"), found);
            }
        }
    }
}

fn operation_templates<'a>(
    operations: &'a [Operation],
    path: &str,
    found: &mut Vec<(String, &'a str)>,
) {
    for (index, operation) in operations.iter().enumerate() {
        let here = format!("{path}[{index}]");
        match operation {
            Operation::Where(condition) => {
                condition_templates(condition, &format!("{here}.where"), found)
            }
            Operation::Map(to) => found.push((format!("{here}.to"), to)),
            Operation::Each(chain) => {
                operation_templates(chain, &format!("{here}.operations"), found)
            }
            _ => {}
        }
    }
}
