use std::collections::BTreeMap;
use std::time::Duration;

use portal_feature::FieldError;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredPath {
    pub path: String,
    pub description: Option<String>,
    pub kind: String,
}

pub trait WidgetReferences: Send + Sync {
    fn call_problems(
        &self,
        workflow: &str,
        inputs: &BTreeMap<String, Option<Value>>,
    ) -> Vec<FieldError>;
    fn script_problem(&self, script: &str) -> Option<String>;
    fn event_fields(&self, automation: &str) -> Option<(String, Vec<String>)>;
    fn declared_paths(&self, workflow: &str) -> Vec<DeclaredPath>;
    fn source_timeout(&self, workflow: &str) -> Duration;
}
