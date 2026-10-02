use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use portal_feature::{EventName, Module};
use serde_json::Value;
use time::OffsetDateTime;
use tokio::sync::Semaphore;

use crate::clients::Runner;
use crate::helpers::manual_event;
use crate::ports::ScriptLibrary;
use crate::services::{
    AutomationSink, Budget, Frame, Secrets, WorkflowRunner, WorkflowTools, bind_inputs,
    input_problem,
};
use crate::types::{
    Automation, Ending, EventValues, Filters, Finished, Invocation, Outcome, Pending, RunControl,
    RunRecord, RunSettings, SourceCall, StepLogging, Tail, Trace, Trigger,
};

pub const AT_ONCE: usize = 4;
pub const WORKFLOW_FAILED: &str = "the workflow failed";
pub const TIMED_OUT: &str = "timed out";
pub const STOPPED: &str = "the run was stopped";

#[derive(Clone)]
pub struct SourceRunner {
    pub sink: Arc<AutomationSink>,
    pub scripts: Arc<dyn ScriptLibrary>,
    pub tools: WorkflowTools,
    pub gate: Arc<Semaphore>,
}

pub struct SourceRun {
    pub data: Result<Value, String>,
    pub record: RunRecord,
}

impl SourceRunner {
    pub fn new(
        sink: Arc<AutomationSink>,
        scripts: Arc<dyn ScriptLibrary>,
        tools: WorkflowTools,
    ) -> SourceRunner {
        SourceRunner {
            sink,
            scripts,
            tools,
            gate: Arc::new(Semaphore::new(AT_ONCE)),
        }
    }

    pub async fn run(&self, widget: &str, title: &str, source: &SourceCall) -> SourceRun {
        let _turn = self.gate.acquire().await;
        let started_at = OffsetDateTime::now_utc();
        let pending = self.pending(widget, title, source, started_at);
        match source {
            SourceCall::Workflow { id, inputs } => {
                self.workflow(pending, id, inputs, started_at).await
            }
            SourceCall::Script {
                script,
                args,
                timeout,
            } => {
                self.script(pending, (script, args, *timeout), started_at)
                    .await
            }
        }
    }

    pub fn record_failure(&self, run: &SourceRun) {
        if run.data.is_err() {
            self.sink.journal.record(run.record.clone());
        }
    }

    fn pending(
        &self,
        widget: &str,
        title: &str,
        source: &SourceCall,
        at: OffsetDateTime,
    ) -> Pending {
        let mut automation = Automation {
            id: SourceCall::run_key(widget),
            title: title.to_string(),
            enabled: true,
            tags: Vec::new(),
            cooldown_seconds: 0,
            trigger: Trigger {
                event: EventName::Manual,
                filters: Filters::default(),
            },
            run: RunSettings::default(),
            workflow: None,
        };
        match source {
            SourceCall::Workflow { id, inputs } => {
                if let Some(workflow) = self.sink.cache.workflow(id) {
                    let called = workflow.manual_run(inputs.clone().into_iter().collect());
                    automation.workflow = called.workflow;
                }
            }
            SourceCall::Script {
                script,
                args,
                timeout,
            } => {
                automation.run = RunSettings {
                    script: script.clone(),
                    args: args.clone(),
                    timeout_seconds: timeout.as_secs(),
                };
            }
        }
        Pending {
            run_id: self.sink.next_id(),
            event: manual_event(&automation, at, &[]),
            automation,
            by: None,
            origin: Vec::new(),
        }
    }

    async fn workflow(
        &self,
        pending: Pending,
        id: &str,
        inputs: &BTreeMap<String, Value>,
        started_at: OffsetDateTime,
    ) -> SourceRun {
        let refused = |reason: String| SourceRun {
            data: Err(reason.clone()),
            record: RunRecord::finished(
                &pending,
                Vec::new(),
                started_at,
                finished(Outcome::Refused, Some(reason), Duration::ZERO),
            ),
        };
        let switches = self.sink.cache.switches();
        if !switches.is_on(Module::Automations) || !switches.is_on(Module::Workflows) {
            return refused(crate::types::Workflow::MODULE_OFF.to_string());
        }
        let Some(workflow) = self.sink.cache.workflow(id) else {
            return refused(format!("no workflow {id}"));
        };
        if !workflow.enabled {
            return refused(crate::types::Workflow::DISABLED.to_string());
        }
        if let Some(missing) = workflow
            .inputs
            .iter()
            .find(|input| input.default.is_none() && !inputs.contains_key(&input.name))
        {
            return refused(format!("the input {} is not given", missing.name));
        }
        let bound = match bind_inputs(&workflow, inputs.clone().into_iter().collect()) {
            Ok(bound) => bound,
            Err(problem) => return refused(input_problem(problem)),
        };
        let began = Instant::now();
        let (_stop, control) = RunControl::new();
        let trace = Arc::new(Mutex::new(Trace::default()));
        let secrets = Arc::new(Secrets::new(self.tools.secrets.clone()));
        let mut frame = Frame::new(
            Arc::new(Invocation::values_of(&pending)),
            bound,
            secrets.clone(),
        );
        let runner = WorkflowRunner {
            workflows: self.sink.cache.workflows(),
            actions: self.tools.actions.clone(),
            http: self.tools.http.clone(),
            scripts: self.scripts.clone(),
            groups: self.sink.groups.clone(),
            budget: Budget::new(
                Duration::from_secs(workflow.timeout_seconds).min(SourceCall::LONGEST_SOURCE),
                control.stop.clone(),
            ),
            trace: trace.clone(),
            starter: self.sink.clone(),
            chain: pending.chain(),
            switches,
            logging: StepLogging::On,
        };
        let ending = runner.run(&workflow, &mut frame).await;
        let elapsed = began.elapsed();
        let (outcome, reason, data) = match ending {
            Ending::Succeeded(reason) => (
                Outcome::Succeeded,
                reason.map(|reason| secrets.mask(&reason)),
                Ok(match frame.outputs.clone() {
                    Some(outputs) => Value::Object(outputs.into_iter().collect()),
                    None => masked(
                        &secrets,
                        Value::Object(frame.vars.clone().into_iter().collect()),
                    ),
                }),
            ),
            Ending::Failed(reason) => (
                Outcome::Failed,
                Some(secrets.mask(&reason)),
                Err(WORKFLOW_FAILED.to_string()),
            ),
            Ending::TimedOut => (Outcome::TimedOut, None, Err(TIMED_OUT.to_string())),
            Ending::Stopped => (Outcome::Stopped, None, Err(STOPPED.to_string())),
        };
        let mut record = RunRecord::finished(
            &pending,
            Vec::new(),
            started_at,
            finished(outcome, reason, elapsed),
        );
        let mut kept = trace.lock().unwrap_or_else(PoisonError::into_inner).clone();
        kept.outputs = frame.outputs.take();
        record.trace = Some(kept);
        record.steps_version = Some(workflow.version.clone());
        SourceRun { data, record }
    }

    async fn script(
        &self,
        pending: Pending,
        (script, args, timeout): (&str, &[String], Duration),
        started_at: OffsetDateTime,
    ) -> SourceRun {
        let refused = |reason: String| SourceRun {
            data: Err(reason.clone()),
            record: RunRecord::finished(
                &pending,
                args.to_vec(),
                started_at,
                finished(Outcome::Refused, Some(reason), Duration::ZERO),
            ),
        };
        if !self.sink.cache.switches().is_on(Module::Automations) {
            return refused("the automations module is off".to_string());
        }
        let program = match self.scripts.resolve(script) {
            Ok(program) => program,
            Err(refusal) => return refused(refusal.message),
        };
        let invocation = Invocation::for_step(
            (program, self.scripts.root()),
            args.to_vec(),
            &EventValues::default(),
            timeout,
        );
        let (_stop, control) = RunControl::new();
        let result = Runner::run(&invocation, self.sink.groups.clone(), control).await;
        let data = match result.outcome {
            Outcome::Succeeded => Ok(read_output(&result.stdout.text())),
            Outcome::TimedOut => Err(TIMED_OUT.to_string()),
            _ => Err(match result.exit_code {
                Some(code) => format!("the script exited {code}"),
                None => "the script failed".to_string(),
            }),
        };
        SourceRun {
            data,
            record: RunRecord::finished(&pending, args.to_vec(), started_at, result),
        }
    }
}

pub fn read_output(text: &str) -> Value {
    serde_json::from_str(text.trim()).unwrap_or_else(|_| Value::String(text.trim().to_string()))
}

fn finished(outcome: Outcome, reason: Option<String>, duration: Duration) -> Finished {
    Finished {
        outcome,
        exit_code: None,
        reason,
        duration,
        stdout: Tail::default(),
        stderr: Tail::default(),
    }
}

fn masked(secrets: &Secrets, data: Value) -> Value {
    let text = data.to_string();
    let hidden = secrets.mask(&text);
    if hidden == text {
        return data;
    }
    serde_json::from_str(&hidden).unwrap_or(Value::Null)
}
