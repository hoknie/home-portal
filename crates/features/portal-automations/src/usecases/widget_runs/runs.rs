use portal_feature::{ApiError, Module, PortalEvent};
use serde_json::Value;
use time::OffsetDateTime;

use crate::helpers::manual_event;
use crate::services::{AutomationSink, SourceRunner, bind_inputs};
use crate::types::{Automation, SourceCall, Workflow};
use std::sync::Arc;

#[derive(Clone)]
pub struct RunWidgetSource {
    pub(crate) runner: SourceRunner,
}

impl RunWidgetSource {
    pub async fn run(&self, widget: &str, title: &str, call: &SourceCall) -> Result<Value, String> {
        let run = self.runner.run(widget, title, call).await;
        self.runner.record_failure(&run);
        run.data
    }

    pub fn available_permits(&self) -> usize {
        self.runner.gate.available_permits()
    }
}

#[derive(Clone)]
pub struct StartWidgetAutomation {
    pub(crate) sink: Arc<AutomationSink>,
}

impl StartWidgetAutomation {
    pub fn run(
        &self,
        widget: &str,
        title: Option<&str>,
        automation: &str,
        fields: &[(String, String)],
        by: &str,
    ) -> Result<u64, ApiError> {
        let cache = &self.sink.cache;
        if !cache.automations_on() {
            return Err(ApiError::Conflict(Workflow::AUTOMATIONS_OFF.to_string()));
        }
        let found = cache
            .find(automation)
            .ok_or_else(|| ApiError::Conflict(format!("no automation {automation}")))?;
        if !found.enabled {
            return Err(ApiError::Conflict(format!(
                "the automation {automation} is disabled"
            )));
        }
        let mut event = manual_event(&found, OffsetDateTime::now_utc(), &[]);
        for (name, value) in fields {
            if let Some(field) = event.fields.iter_mut().find(|(field, _)| field == name) {
                field.1 = PortalEvent::clean(value);
            }
        }
        let tagged = Automation {
            id: SourceCall::run_key(widget),
            title: title.map_or(found.title.clone(), str::to_string),
            ..found
        };
        Ok(self
            .sink
            .admit(&tagged, event, (Some(by.to_string()), Vec::new())))
    }
}

#[derive(Clone)]
pub struct StartWidgetWorkflow {
    pub(crate) sink: Arc<AutomationSink>,
}

impl StartWidgetWorkflow {
    pub fn run(
        &self,
        widget: &str,
        title: Option<&str>,
        workflow: &str,
        inputs: Vec<(String, Value)>,
        by: &str,
    ) -> Result<u64, ApiError> {
        let cache = &self.sink.cache;
        if !cache.automations_on() {
            return Err(ApiError::Conflict(Workflow::AUTOMATIONS_OFF.to_string()));
        }
        let found = cache
            .workflow(workflow)
            .ok_or_else(|| ApiError::Conflict(format!("no workflow {workflow}")))?;
        if !cache.switches().is_on(Module::Workflows) {
            return Err(ApiError::Conflict(Workflow::MODULE_OFF.to_string()));
        }
        if !found.enabled {
            return Err(ApiError::Conflict(Workflow::DISABLED.to_string()));
        }
        let bound = bind_inputs(&found, inputs)
            .map_err(|(name, message)| ApiError::Conflict(format!("inputs.{name} {message}")))?;
        let mut automation = found.manual_run(bound.into_iter().collect());
        automation.id = SourceCall::run_key(widget);
        if let Some(title) = title {
            automation.title = title.to_string();
        }
        Ok(self.sink.run_now(&automation, by))
    }
}
