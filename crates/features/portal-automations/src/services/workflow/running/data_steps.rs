use std::time::Duration;

use serde_json::Value;

use super::runner::WorkflowRunner;
use crate::services::workflow::evaluating::{
    Frame, oversized, render_json, render_keys, render_number, render_text, render_value, text_of,
};
use crate::types::{Ending, NumberSetting, SetValue, StepReport, Workflow};

pub const STOPPED_BY_WORKFLOW: &str = "stopped by the workflow";

pub fn run_set(variable: &str, value: &SetValue, frame: &mut Frame) -> StepReport {
    let rendered = match value {
        SetValue::Text(template) => render_value(template, frame),
        SetValue::Json(template) => render_json(template, frame),
        SetValue::List(items) => items
            .iter()
            .map(|item| render_value(item, frame))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        SetValue::Object(entries) => render_keys(entries, frame).and_then(|entries| {
            entries
                .into_iter()
                .map(|(key, item)| render_value(item, frame).map(|value| (key, value)))
                .collect::<Result<serde_json::Map<_, _>, _>>()
                .map(Value::Object)
        }),
    };
    match rendered {
        Ok(value) if oversized(variable, &value).is_some() => {
            StepReport::failed(oversized(variable, &value).unwrap_or_default())
        }
        Ok(value) => {
            frame.vars.insert(variable.to_string(), value.clone());
            let detail = format!("{variable} = {}", text_of(&value));
            let line = format!("{variable} = {value}");
            StepReport::done(WorkflowRunner::step_result(&[("value", value)]), detail).logged(line)
        }
        Err(message) => StepReport::failed(message),
    }
}

pub async fn run_wait(
    runner: &WorkflowRunner,
    seconds: &NumberSetting,
    frame: &Frame,
) -> StepReport {
    let seconds = match render_number(seconds, (1, Workflow::LONGEST_WAIT), "seconds", frame) {
        Ok(seconds) => seconds,
        Err(message) => return StepReport::failed(message),
    };
    runner.publish(frame, Some(seconds));
    match runner.budget.pause(Duration::from_secs(seconds)).await {
        Ok(()) => StepReport::done(Value::Null, format!("{seconds} s"))
            .logged(format!("waited {seconds} s")),
        Err(ending) => StepReport::ended(ending),
    }
}

pub fn run_stop(succeeded: bool, reason: Option<&str>, frame: &Frame) -> StepReport {
    let reason = match reason
        .map(|template| render_text(template, frame))
        .transpose()
    {
        Ok(reason) => reason.filter(|reason| !reason.trim().is_empty()),
        Err(message) => return StepReport::failed(message),
    };
    let line = format!(
        "stopped the run as {}{}",
        if succeeded { "succeeded" } else { "failed" },
        reason
            .as_deref()
            .map(|reason| format!(": {reason}"))
            .unwrap_or_default()
    );
    let report = if succeeded {
        StepReport::ended(Ending::Succeeded(reason))
    } else {
        StepReport::ended(Ending::Failed(
            reason.unwrap_or_else(|| STOPPED_BY_WORKFLOW.to_string()),
        ))
    };
    report.logged(line)
}
