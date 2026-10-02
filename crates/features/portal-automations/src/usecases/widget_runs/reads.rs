use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::services::{AutomationSink, DeclaredPath, declared_paths};
use crate::types::{SourceCall, WidgetTemplates};

#[derive(Clone, Copy, Default)]
pub struct OpenWidgetTemplates;

impl OpenWidgetTemplates {
    pub fn run(
        &self,
        data: &Value,
        id: &str,
        title: Option<&str>,
        fetched_at: Option<String>,
    ) -> WidgetTemplates {
        WidgetTemplates::of(data, id, title, fetched_at)
    }
}

#[derive(Clone)]
pub struct ReadEventFields {
    pub(crate) sink: Arc<AutomationSink>,
}

impl ReadEventFields {
    pub fn run(&self, automation: &str) -> Option<(String, Vec<String>)> {
        self.sink
            .cache
            .automations()
            .iter()
            .find(|found| found.id == automation)
            .map(|found| {
                (
                    found.trigger.event.name().to_string(),
                    found
                        .trigger
                        .event
                        .fields()
                        .iter()
                        .map(|field| field.to_string())
                        .collect(),
                )
            })
    }
}

#[derive(Clone)]
pub struct ReadDeclaredPaths {
    pub(crate) sink: Arc<AutomationSink>,
}

impl ReadDeclaredPaths {
    pub fn run(&self, workflow: &str) -> Vec<DeclaredPath> {
        self.sink
            .cache
            .workflow(workflow)
            .map(|found| declared_paths(&found))
            .unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct ReadSourceTimeout {
    pub(crate) sink: Arc<AutomationSink>,
}

impl ReadSourceTimeout {
    pub fn run(&self, workflow: &str) -> Duration {
        self.sink
            .cache
            .workflow(workflow)
            .map_or(SourceCall::LONGEST_SOURCE, |found| {
                Duration::from_secs(found.timeout_seconds).min(SourceCall::LONGEST_SOURCE)
            })
    }
}
