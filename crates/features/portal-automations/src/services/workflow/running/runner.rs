use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde_json::Value;
use time::OffsetDateTime;

use super::action_steps::run_action;
use super::automation_step::run_automation;
use super::budget::Budget;
use super::data_steps::{run_set, run_stop, run_wait};
use super::flow_steps::{run_call, run_if, run_loop, run_parallel};
use super::http_step::run_http;
use super::join::Pending;
use super::script_step::run_script;
use super::transform_step::run_transform;
use crate::clients::{GroupRegistry, HttpClient};
use crate::ports::{AutomationStarter, PortalActions};
use crate::services::ScriptsDirectory;
use portal_feature::ModuleSwitches;

use crate::services::workflow::checking::templates_of;
use crate::services::workflow::evaluating::{Frame, collected, collector, placeholders_in};
use crate::types::{
    Ending, EntryEnd, Flow, Place, Step, StepKind, StepLog, StepLogging, StepOutcome, StepReport,
    Trace, TraceEntry, Workflow,
};

pub const PORTAL: &str = "portal";

pub struct WorkflowRunner {
    pub workflows: Arc<Vec<Workflow>>,
    pub actions: Arc<dyn PortalActions>,
    pub http: HttpClient,
    pub scripts: ScriptsDirectory,
    pub groups: Arc<dyn GroupRegistry>,
    pub budget: Budget,
    pub trace: Arc<Mutex<Trace>>,
    pub starter: Arc<dyn AutomationStarter>,
    pub chain: Vec<String>,
    pub switches: ModuleSwitches,
    pub logging: StepLogging,
}

impl WorkflowRunner {
    pub async fn run(&self, workflow: &Workflow, frame: &mut Frame) -> Ending {
        match self.run_steps(&workflow.steps, frame, &Place::root()).await {
            Flow::Continue => Ending::Succeeded(None),
            Flow::End(ending) => ending,
        }
    }

    pub fn run_steps<'a>(
        &'a self,
        steps: &'a [Step],
        frame: &'a mut Frame,
        place: &'a Place,
    ) -> Pending<'a, Flow> {
        Box::pin(async move {
            for (index, step) in steps.iter().enumerate() {
                let here = Place {
                    path: format!("{}[{index}]", place.path),
                    ..place.clone()
                };
                if let Flow::End(ending) = self.run_step(step, frame, &here).await {
                    return Flow::End(ending);
                }
            }
            Flow::Continue
        })
    }

    async fn run_step(&self, step: &Step, frame: &mut Frame, place: &Place) -> Flow {
        if let Err(ending) = self.budget.admit() {
            return Flow::End(ending);
        }
        let index = self.trace().start(TraceEntry {
            path: place.path.clone(),
            step: step.id.clone(),
            label: step.label.clone(),
            kind: step.kind.name().to_string(),
            iteration: place.iteration,
            outcome: StepOutcome::Running,
            started_at: OffsetDateTime::now_utc(),
            duration: Duration::ZERO,
            detail: String::new(),
            output: None,
            shape: None,
            log: StepLog::default(),
            item: place.item.clone(),
            level: match &step.kind {
                StepKind::Log { level, .. } => Some(*level),
                _ => None,
            },
        });
        let own = (self.logging == StepLogging::On).then(collector);
        let parent = std::mem::replace(&mut frame.rendered, own.clone());
        let report = match self.refresh_portal(step, frame).await {
            Ok(()) => self.dispatch(step, frame, place).await,
            Err(message) => StepReport::failed(message),
        };
        frame.rendered = parent;
        let log = match &own {
            Some(own) => {
                let mut log = collected(own);
                log.extend_lines(report.log.clone());
                log.masked(|text| frame.secrets.mask(text))
            }
            None => StepLog::default(),
        };
        if !report.result.is_null() {
            frame.steps.insert(step.id.clone(), report.result.clone());
        }
        let outcome = match &report.flow {
            Flow::Continue | Flow::End(Ending::Succeeded(_)) => StepOutcome::Succeeded,
            Flow::End(Ending::Failed(_)) => StepOutcome::Failed,
            Flow::End(Ending::TimedOut) => StepOutcome::TimedOut,
            Flow::End(Ending::Stopped) => StepOutcome::Stopped,
        };
        let detail = frame.secrets.mask(&report.detail);
        let output = report
            .output
            .as_deref()
            .map(|output| frame.secrets.mask(output));
        let shape = report
            .shape
            .as_deref()
            .map(|shape| frame.secrets.mask(shape));
        self.trace().finish(
            index,
            EntryEnd {
                outcome,
                detail,
                output,
                shape,
                log,
            },
            OffsetDateTime::now_utc(),
        );
        report.flow
    }

    async fn dispatch(&self, step: &Step, frame: &mut Frame, place: &Place) -> StepReport {
        match &step.kind {
            StepKind::If { .. } => run_if(self, step, frame, place).await,
            StepKind::Loop { .. } => run_loop(self, step, frame, place).await,
            StepKind::Parallel { branches } => run_parallel(self, branches, frame, place).await,
            StepKind::Call { .. } => run_call(self, step, frame, place).await,
            StepKind::Stop { succeeded, reason } => run_stop(*succeeded, reason.as_deref(), frame),
            StepKind::Set { variable, value } => run_set(variable, value, frame),
            StepKind::Wait { seconds } => run_wait(self, *seconds).await,
            StepKind::Nothing => StepReport::done(Value::Null, ""),
            StepKind::Automation {
                automation,
                fields,
                wait,
            } => run_automation(self, (automation, fields, *wait), frame).await,
            StepKind::Transform { input, operations } => run_transform(input, operations, frame),
            StepKind::Http(http) => run_http(self, http, frame).await,
            StepKind::Script { run, env, stdin } => {
                run_script(self, (run, env, stdin.as_deref()), frame).await
            }
            _ => run_action(self, &step.kind, frame).await,
        }
    }

    async fn refresh_portal(&self, step: &Step, frame: &mut Frame) -> Result<(), String> {
        let named = templates_of(step).iter().any(|(_, template)| {
            placeholders_in(template)
                .iter()
                .any(|placeholder| placeholder.name.starts_with(PORTAL))
        });
        if !named {
            return Ok(());
        }
        let state = self.actions.state().await?;
        frame.portal = Some(state.value_with(self.switches));
        Ok(())
    }

    pub fn trace(&self) -> std::sync::MutexGuard<'_, Trace> {
        self.trace.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn step_result(fields: &[(&str, Value)]) -> Value {
        Value::Object(
            fields
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect(),
        )
    }
}
