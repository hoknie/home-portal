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
                Ok(Ok(detail)) => {
                    let line = format!(
                        "sent to {}: {detail}",
                        channel.as_deref().unwrap_or("every enabled channel")
                    );
                    StepReport::done(Value::Null, detail).logged(line)
                }
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
                )
                .logged(format!(
                    "{service} → {}{}",
                    probe.state,
                    probe
                        .latency_milliseconds
                        .map(|latency| format!(" in {latency} ms"))
                        .unwrap_or_default()
                )),
                Ok(Err(message)) => {
                    StepReport::failed(message).with_log(not_found_hint(runner, &service).await)
                }
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
                )
                .logged(format!("{service} → {}", status.state)),
                Ok(Err(message)) => {
                    StepReport::failed(message).with_log(not_found_hint(runner, &service).await)
                }
                Err(ending) => StepReport::ended(ending),
            }
        }
        StepKind::Log { message, level } => match render_text(message, frame) {
            Ok(message) => StepReport::done(
                WorkflowRunner::step_result(&[("message", Value::from(message.clone()))]),
                message.clone(),
            )
            .logged(format!("[{}] {message}", level.name())),
            Err(message) => StepReport::failed(message),
        },
        other => StepReport::failed(format!("{} steps are not available yet", other.name())),
    }
}

pub const MOST_KNOWN_IDS: usize = 20;

async fn not_found_hint(runner: &WorkflowRunner, service: &str) -> Vec<String> {
    let Ok(state) = runner.actions.state().await else {
        return Vec::new();
    };
    if state.services.iter().any(|known| known.id == service) {
        return Vec::new();
    }
    let mut lines = Vec::new();
    if let Some(named) = state
        .services
        .iter()
        .find(|known| known.name.eq_ignore_ascii_case(service.trim()))
    {
        lines.push(format!(
            "\"{service}\" is the name of the service {id}; use {{{{portal.services.{id}.id}}}}",
            id = named.id
        ));
    }
    let ids: Vec<&str> = state
        .services
        .iter()
        .take(MOST_KNOWN_IDS)
        .map(|known| known.id.as_str())
        .collect();
    let more = state.services.len().saturating_sub(MOST_KNOWN_IDS);
    lines.push(format!(
        "known services: {}{}",
        ids.join(", "),
        if more > 0 {
            format!(" and {more} more")
        } else {
            String::new()
        }
    ));
    lines
}
