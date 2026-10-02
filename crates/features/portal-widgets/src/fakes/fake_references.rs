use std::collections::BTreeMap;
use std::time::Duration;

use portal_feature::FieldError;
use serde_json::Value;

use crate::ports::{DeclaredPath, WidgetReferences};

#[derive(Default)]
pub struct FakeReferences {
    pub workflows: BTreeMap<String, Vec<String>>,
    pub scripts: Vec<String>,
    pub automations: BTreeMap<String, Vec<String>>,
    pub paths: BTreeMap<String, Vec<DeclaredPath>>,
}

impl WidgetReferences for FakeReferences {
    fn call_problems(
        &self,
        workflow: &str,
        inputs: &BTreeMap<String, Option<Value>>,
    ) -> Vec<FieldError> {
        let Some(declared) = self.workflows.get(workflow) else {
            return vec![FieldError::new(
                "workflow",
                format!("names {workflow:?}, which is not a workflow"),
            )];
        };
        inputs
            .keys()
            .filter(|name| !declared.contains(name))
            .map(|name| {
                FieldError::new(format!("inputs.{name}"), "is not an input of the workflow")
            })
            .collect()
    }

    fn script_problem(&self, script: &str) -> Option<String> {
        (!self.scripts.iter().any(|known| known == script))
            .then(|| format!("{script} is not in the scripts folder"))
    }

    fn event_fields(&self, automation: &str) -> Option<(String, Vec<String>)> {
        self.automations
            .get(automation)
            .map(|fields| ("manual".to_string(), fields.clone()))
    }

    fn declared_paths(&self, workflow: &str) -> Vec<DeclaredPath> {
        self.paths.get(workflow).cloned().unwrap_or_default()
    }

    fn source_timeout(&self, _workflow: &str) -> Duration {
        Duration::from_secs(60)
    }
}
