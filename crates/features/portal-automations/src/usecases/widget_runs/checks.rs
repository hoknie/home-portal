use std::collections::BTreeMap;
use std::sync::Arc;

use portal_feature::FieldError;
use serde_json::Value;

use crate::ports::ScriptLibrary;
use crate::services::{AutomationSink, call_problems, template_problem};
use crate::types::{InputValue, ValueType, WorkflowCall};

#[derive(Clone, Copy, Default)]
pub struct CheckWidgetTemplate;

impl CheckWidgetTemplate {
    pub fn run(
        &self,
        template: &str,
        allows: &dyn Fn(&str) -> Result<ValueType, String>,
    ) -> Option<String> {
        template_problem(template, allows)
    }
}

#[derive(Clone)]
pub struct CheckWidgetCall {
    pub(crate) sink: Arc<AutomationSink>,
}

impl CheckWidgetCall {
    pub fn run(&self, workflow: &str, inputs: &BTreeMap<String, Option<Value>>) -> Vec<FieldError> {
        let call = WorkflowCall {
            id: workflow.to_string(),
            inputs: inputs
                .iter()
                .map(|(name, value)| {
                    let given = match value {
                        Some(literal) => InputValue::Literal(literal.clone()),
                        None => InputValue::Template(String::new()),
                    };
                    (name.clone(), given)
                })
                .collect(),
        };
        call_problems(&call, &self.sink.cache.workflows())
    }
}

#[derive(Clone)]
pub struct CheckWidgetScript {
    pub(crate) scripts: Arc<dyn ScriptLibrary>,
}

impl CheckWidgetScript {
    pub fn run(&self, script: &str) -> Option<String> {
        self.scripts
            .resolve(script)
            .err()
            .map(|refusal| refusal.message)
    }
}
