use std::time::Duration;

use serde_json::Value;

use super::runner::WorkflowRunner;
use crate::services::workflow::evaluating::{Frame, render_text};
use crate::types::{Outcome, StepReport};

pub const POLL: Duration = Duration::from_millis(100);

pub async fn run_automation(
    runner: &WorkflowRunner,
    (automation, fields, wait): (&str, &[(String, String)], bool),
    frame: &Frame,
) -> StepReport {
    let mut rendered = Vec::new();
    for (name, template) in fields {
        match render_text(template, frame) {
            Ok(value) => rendered.push((name.clone(), value)),
            Err(message) => return StepReport::failed(message),
        }
    }
    let run_id = match runner.starter.start(automation, &rendered, &runner.chain) {
        Ok(run_id) => run_id,
        Err(message) => return StepReport::failed(message),
    };
    let id = Value::String(run_id.to_string());
    if let Some((Outcome::Skipped | Outcome::Refused, reason)) = runner.starter.outcome_of(run_id) {
        return StepReport::failed(format!(
            "{automation} was not run: {}",
            reason.unwrap_or_default()
        ));
    }
    let queued = format!("queued {automation} as run {run_id}");
    if !wait {
        return StepReport::done(
            WorkflowRunner::step_result(&[("run_id", id)]),
            format!("{automation} queued as run {run_id}"),
        )
        .logged(queued);
    }
    runner.publish(frame, None);
    let waited = runner
        .budget
        .guard(async {
            loop {
                if let Some(found) = runner.starter.outcome_of(run_id) {
                    return found;
                }
                tokio::time::sleep(POLL).await;
            }
        })
        .await;
    let (outcome, reason) = match waited {
        Ok(found) => found,
        Err(ending) => return StepReport::ended(ending),
    };
    let result =
        WorkflowRunner::step_result(&[("run_id", id), ("outcome", Value::from(outcome.name()))]);
    let detail = format!("{automation} run {run_id}: {}", outcome.name());
    let ended = format!("run {run_id} ended {}", outcome.name());
    let report = match outcome {
        Outcome::Succeeded => StepReport::done(result, detail),
        _ => StepReport {
            result,
            ..StepReport::failed(match reason {
                Some(reason) => format!("{detail} ({reason})"),
                None => detail,
            })
        },
    };
    report.with_log(vec![queued, ended])
}
