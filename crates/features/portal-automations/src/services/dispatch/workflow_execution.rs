use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use portal_feature::Module;
use serde_json::Value;
use time::OffsetDateTime;

use super::AutomationSink;
use crate::ports::ScriptLibrary;
use crate::services::{
    Budget, Frame, Secrets, WorkflowRunner, WorkflowTools, bind_inputs, input_problem,
};
use crate::types::{
    Ending, Finished, InputValue, Invocation, Outcome, Pending, RunControl, RunRecord, StepLogging,
    Tail, Trace, Workflow, WorkflowCall,
};

pub const WORKFLOWS_OFF: &str = "workflows-off";

pub async fn execute_workflow(
    sink: Arc<AutomationSink>,
    (scripts, tools): (&Arc<dyn ScriptLibrary>, &WorkflowTools),
    pending: Pending,
    call: &WorkflowCall,
) {
    let started_at = OffsetDateTime::now_utc();
    let began = Instant::now();
    let control = sink
        .active
        .control(pending.run_id)
        .unwrap_or_else(|| RunControl::new().1);
    let trace = sink
        .active
        .trace(pending.run_id)
        .unwrap_or_else(|| Arc::new(Mutex::new(Trace::default())));
    let workflow = sink.cache.workflow(&call.id);
    let steps_version = workflow.as_ref().map(|workflow| workflow.version.clone());
    let refusal = if !sink.cache.switches().is_on(Module::Workflows) {
        Some(format!("{WORKFLOWS_OFF}: {}", Workflow::MODULE_OFF))
    } else {
        match &workflow {
            None => Some(format!("no workflow {}", call.id)),
            Some(workflow) if !workflow.enabled => {
                Some(format!("the workflow {} is disabled", workflow.id))
            }
            Some(_) => None,
        }
    };
    let event = Arc::new(Invocation::values_of(&pending));
    let finished = match (refusal, workflow) {
        (Some(reason), _) => finished_as(Outcome::Refused, Some(reason), Duration::ZERO),
        (None, None) => finished_as(
            Outcome::Refused,
            Some(format!("no workflow {}", call.id)),
            Duration::ZERO,
        ),
        (None, Some(workflow)) => {
            let given: Vec<(String, Value)> = call
                .inputs
                .iter()
                .map(|(name, value)| {
                    let value = match value {
                        InputValue::Template(template) => event.resolve(template),
                        InputValue::Literal(literal) => literal.clone(),
                    };
                    (name.clone(), value)
                })
                .collect();
            sink.active.started(pending.run_id, Vec::new(), started_at);
            let inputs = match bind_inputs(&workflow, given) {
                Ok(inputs) => inputs,
                Err(problem) => {
                    let finished = finished_as(
                        Outcome::Failed,
                        Some(input_problem(problem)),
                        Duration::ZERO,
                    );
                    let record = RunRecord::finished(&pending, Vec::new(), started_at, finished);
                    sink.journal.record(record);
                    sink.active.remove(pending.run_id);
                    return;
                }
            };
            let secrets = Arc::new(Secrets::new(tools.secrets.clone()));
            let mut frame = Frame::new(event.clone(), inputs, secrets.clone());
            let runner = WorkflowRunner {
                workflows: sink.cache.workflows(),
                actions: tools.actions.clone(),
                http: tools.http.clone(),
                scripts: scripts.clone(),
                groups: sink.groups.clone(),
                budget: Budget::new(
                    Duration::from_secs(workflow.timeout_seconds),
                    control.stop.clone(),
                ),
                trace: trace.clone(),
                starter: sink.clone(),
                chain: pending.chain(),
                switches: sink.cache.switches(),
                logging: StepLogging::On,
            };
            let ending = runner.run(&workflow, &mut frame).await;
            let elapsed = began.elapsed();
            match ending {
                Ending::Succeeded(reason) => finished_as(
                    Outcome::Succeeded,
                    reason.map(|reason| secrets.mask(&reason)),
                    elapsed,
                ),
                Ending::Failed(reason) => {
                    finished_as(Outcome::Failed, Some(secrets.mask(&reason)), elapsed)
                }
                Ending::TimedOut => finished_as(Outcome::TimedOut, None, elapsed),
                Ending::Stopped => finished_as(Outcome::Stopped, None, elapsed),
            }
        }
    };
    let mut record = RunRecord::finished(&pending, Vec::new(), started_at, finished);
    record.trace = Some(trace.lock().unwrap_or_else(PoisonError::into_inner).clone());
    record.steps_version = steps_version;
    if record.result.outcome == Outcome::Stopped {
        let by = sink.active.stopped_by(pending.run_id).unwrap_or_default();
        record.result.reason = Some(RunRecord::stopped_reason(&by));
    }
    sink.journal.record(record);
    sink.active.remove(pending.run_id);
}

fn finished_as(outcome: Outcome, reason: Option<String>, duration: Duration) -> Finished {
    Finished {
        outcome,
        exit_code: None,
        reason,
        duration,
        stdout: Tail::default(),
        stderr: Tail::default(),
    }
}
