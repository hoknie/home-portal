use serde_json::Value;

use super::runner::WorkflowRunner;
use crate::services::workflow::evaluating::{Frame, render_text};
use crate::types::{StepKind, StepReport};

pub async fn run_action(runner: &WorkflowRunner, kind: &StepKind, frame: &mut Frame) -> StepReport {
    match kind {
        StepKind::Notify {
            channel,
            title,
            text,
        } => {
            let text = match render_text(text, frame) {
                Ok(text) => text,
                Err(message) => return StepReport::failed(message),
            };
            let title = match title
                .as_deref()
                .map(|title| render_text(title, frame))
                .transpose()
            {
                Ok(title) => title.unwrap_or_default(),
                Err(message) => return StepReport::failed(message),
            };
            match runner
                .budget
                .guard(runner.actions.notify(channel.as_deref(), &title, &text))
                .await
            {
                Ok(Ok(detail)) => StepReport::done(Value::Null, detail),
                Ok(Err(message)) => StepReport::failed(message),
                Err(ending) => StepReport::ended(ending),
            }
        }
        StepKind::Probe { service } => {
            let service = match render_text(service, frame) {
                Ok(service) => service,
                Err(message) => return StepReport::failed(message),
            };
            match runner.budget.guard(runner.actions.probe(&service)).await {
                Ok(Ok(probe)) => StepReport::done(
                    WorkflowRunner::step_result(&[
                        ("state", Value::from(probe.state.clone())),
                        (
                            "latency_milliseconds",
                            probe.latency_milliseconds.map_or(Value::Null, Value::from),
                        ),
                        (
                            "diagnosis",
                            probe.diagnosis.clone().map_or(Value::Null, Value::from),
                        ),
                    ]),
                    format!("{service}: {}", probe.state),
                ),
                Ok(Err(message)) => StepReport::failed(message),
                Err(ending) => StepReport::ended(ending),
            }
        }
        StepKind::Status { service } => {
            let service = match render_text(service, frame) {
                Ok(service) => service,
                Err(message) => return StepReport::failed(message),
            };
            match runner.budget.guard(runner.actions.status(&service)).await {
                Ok(Ok(status)) => StepReport::done(
                    WorkflowRunner::step_result(&[
                        ("state", Value::from(status.state.clone())),
                        (
                            "since",
                            status.since.clone().map_or(Value::Null, Value::from),
                        ),
                    ]),
                    format!("{service}: {}", status.state),
                ),
                Ok(Err(message)) => StepReport::failed(message),
                Err(ending) => StepReport::ended(ending),
            }
        }
        other => StepReport::failed(format!("{} steps are not available yet", other.name())),
    }
}
