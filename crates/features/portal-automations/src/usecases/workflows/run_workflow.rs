use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use portal_feature::{ApiError, FieldError, Module};

use serde_json::Value;

use crate::services::{AutomationSink, bind_inputs};
use crate::types::Workflow;

#[derive(Clone)]
pub struct RunWorkflow {
    sink: Arc<AutomationSink>,
    manual_runs: Arc<Mutex<HashMap<String, Instant>>>,
}

impl RunWorkflow {
    pub const EVERY: Duration = Duration::from_secs(5);

    pub fn new(
        sink: Arc<AutomationSink>,
        manual_runs: Arc<Mutex<HashMap<String, Instant>>>,
    ) -> RunWorkflow {
        RunWorkflow { sink, manual_runs }
    }

    pub fn run(
        &self,
        id: &str,
        inputs: BTreeMap<String, Value>,
        by: &str,
    ) -> Result<u64, ApiError> {
        let cache = &self.sink.cache;
        let workflow = cache
            .workflow(id)
            .ok_or(ApiError::NotFound(Workflow::UNKNOWN))?;
        if !cache.automations_on() || !cache.switches().is_on(Module::Workflows) {
            return Err(ApiError::Conflict(Workflow::MODULE_OFF.to_string()));
        }
        if !workflow.enabled {
            return Err(ApiError::Conflict(Workflow::DISABLED.to_string()));
        }
        let undeclared: Vec<FieldError> = inputs
            .keys()
            .filter(|name| !workflow.declares(name))
            .map(|name| FieldError::new(format!("inputs.{name}"), Workflow::UNDECLARED_INPUT))
            .collect();
        if !undeclared.is_empty() {
            return Err(ApiError::Invalid(undeclared));
        }
        let values: Vec<(String, Value)> = inputs.into_iter().collect();
        let bound = bind_inputs(&workflow, values).map_err(|(name, message)| {
            ApiError::Invalid(vec![FieldError::new(format!("inputs.{name}"), message)])
        })?;
        self.throttle(&Workflow::manual_key(id))?;
        Ok(self
            .sink
            .run_now(&workflow.manual_run(bound.into_iter().collect()), by))
    }

    fn throttle(&self, key: &str) -> Result<(), ApiError> {
        let mut manual = self
            .manual_runs
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let now = Instant::now();
        if let Some(last) = manual.get(key) {
            let since = now.saturating_duration_since(*last);
            if since < Self::EVERY {
                return Err(ApiError::TooManyRequests {
                    retry_after_seconds: (Self::EVERY - since).as_secs().max(1),
                });
            }
        }
        manual.insert(key.to_string(), now);
        Ok(())
    }
}
