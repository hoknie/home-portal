use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;

use super::runner::WorkflowRunner;
use crate::clients::Runner;
use crate::services::workflow::evaluating::{Frame, render_text, render_value};
use crate::types::{Ending, Invocation, Outcome, RunControl, RunSettings, StepReport, Tail};

pub async fn run_script(
    runner: &WorkflowRunner,
    (run, env, stdin): (&RunSettings, &[(String, String)], Option<&str>),
    frame: &Frame,
) -> StepReport {
    let program = match runner.scripts.resolve(&run.script) {
        Ok(program) => program,
        Err(refusal) => return StepReport::failed(refusal.message),
    };
    let mut arguments = Vec::new();
    for template in &run.args {
        match render_text(template, frame) {
            Ok(argument) => arguments.push(argument),
            Err(message) => return StepReport::failed(message),
        }
    }
    let remaining = runner.budget.remaining();
    let own = Duration::from_secs(run.timeout_seconds);
    let mut invocation = Invocation::for_step(
        (program, runner.scripts.root()),
        arguments,
        &frame.event,
        own.min(remaining),
    );
    for (name, template) in env {
        match render_text(template, frame) {
            Ok(value) => invocation.environment.push((name.clone(), value)),
            Err(message) => return StepReport::failed(message),
        }
    }
    if let Some(template) = stdin {
        match render_value(template, frame) {
            Ok(value) => invocation.input = value.to_string(),
            Err(message) => return StepReport::failed(message),
        }
    }
    let control = RunControl {
        stdout: Arc::new(Mutex::new(Tail::default())),
        stderr: Arc::new(Mutex::new(Tail::default())),
        stop: runner.budget.stop_signal(),
        grace: RunControl::STOP_GRACE,
    };
    let finished = Runner::run(&invocation, runner.groups.clone(), control).await;
    let stdout = finished.stdout.text();
    let stderr = finished.stderr.text();
    let result = WorkflowRunner::step_result(&[
        (
            "exit_code",
            finished.exit_code.map_or(Value::Null, Value::from),
        ),
        ("stdout", Value::from(stdout.clone())),
        ("stderr", Value::from(stderr.clone())),
    ]);
    let output = Some(format!("{stdout}{stderr}"));
    let line = format!(
        "{} → {}",
        std::iter::once(run.script.as_str())
            .chain(invocation.arguments.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" "),
        finished
            .exit_code
            .map(|code| format!("exit {code}"))
            .or_else(|| finished.reason.clone())
            .unwrap_or_else(|| finished.outcome.name().to_string())
    );
    let detail = match finished.exit_code {
        Some(code) => format!("{} exited {code}", run.script),
        None => finished
            .reason
            .clone()
            .unwrap_or_else(|| run.script.clone()),
    };
    let flow_report = match finished.outcome {
        Outcome::Succeeded => StepReport::done(result, detail),
        Outcome::Stopped => StepReport::ended(Ending::Stopped),
        Outcome::TimedOut if own > remaining => StepReport::ended(Ending::TimedOut),
        Outcome::TimedOut => StepReport {
            result,
            ..StepReport::failed(format!(
                "{} timed out after {} s",
                run.script, run.timeout_seconds
            ))
        },
        _ => StepReport {
            result,
            ..StepReport::failed(detail)
        },
    };
    StepReport {
        output,
        ..flow_report.logged(line)
    }
}
