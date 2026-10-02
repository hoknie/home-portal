use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use portal_feature::ApiError;
use serde_json::Value;

use crate::ports::{Started, WidgetRuns};
use crate::types::WidgetSource;

#[derive(Default)]
pub struct FakeRuns {
    pub answers: BTreeMap<String, Result<Value, String>>,
    pub sources: Mutex<Vec<String>>,
    pub started: Mutex<Vec<String>>,
    pub next: AtomicU64,
}

fn name_of(source: &WidgetSource) -> String {
    match source {
        WidgetSource::Workflow { id, .. } => id.clone(),
        WidgetSource::Script { script, .. } => script.clone(),
    }
}

#[async_trait]
impl WidgetRuns for FakeRuns {
    async fn run_source(
        &self,
        _widget: &str,
        _title: &str,
        source: &WidgetSource,
    ) -> Result<Value, String> {
        let name = name_of(source);
        self.sources.lock().unwrap().push(name.clone());
        self.answers.get(&name).cloned().unwrap_or(Ok(Value::Null))
    }

    fn start_automation(
        &self,
        started: Started<'_>,
        automation: &str,
        fields: &[(String, String)],
    ) -> Result<u64, ApiError> {
        self.started.lock().unwrap().push(format!(
            "{} automation {automation} by {} {fields:?}",
            started.widget, started.by
        ));
        Ok(self.next.fetch_add(1, Ordering::SeqCst) + 1)
    }

    fn start_workflow(
        &self,
        started: Started<'_>,
        workflow: &str,
        inputs: Vec<(String, Value)>,
    ) -> Result<u64, ApiError> {
        self.started.lock().unwrap().push(format!(
            "{} workflow {workflow} by {} {inputs:?}",
            started.widget, started.by
        ));
        Ok(self.next.fetch_add(1, Ordering::SeqCst) + 1)
    }
}
