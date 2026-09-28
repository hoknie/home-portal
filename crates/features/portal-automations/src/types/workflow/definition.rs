use portal_feature::EventName;

use super::Step;
use serde_json::Value;

use crate::types::{
    Automation, Filters, InputDeclaration, InputValue, RunSettings, Trigger, WorkflowCall,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub timeout_seconds: u64,
    pub inputs: Vec<InputDeclaration>,
    pub steps: Vec<Step>,
}

impl Workflow {
    pub const SECTION: &'static str = "workflows";
    pub const UNKNOWN: &'static str = "no such workflow";
    pub const TAKEN_ID: &'static str = "is used by another workflow";
    pub const RESERVED_IDS: [&'static str; 2] = ["catalogue", "runs"];
    pub const DEFAULT_TIMEOUT: u64 = 300;
    pub const LONGEST_TIMEOUT: u64 = 3600;
    pub const LONGEST_ID: usize = 63;
    pub const DEEPEST_NESTING: usize = 8;
    pub const DEEPEST_CALLS: usize = 4;
    pub const MOST_STEPS_RUN: usize = 1000;
    pub const MOST_ITERATIONS: u32 = 100;
    pub const MOST_BRANCHES: usize = 4;
    pub const FEWEST_BRANCHES: usize = 2;
    pub const LONGEST_WAIT: u64 = 3600;
    pub const LARGEST_VALUE: usize = 64 * 1024;
    pub const MANUAL_PREFIX: &'static str = "workflow:";
    pub const DISABLED: &'static str = "the workflow is disabled";
    pub const MODULE_OFF: &'static str = "the workflows module is off";
    pub const UNDECLARED_INPUT: &'static str = "is not an input of the workflow";

    pub fn manual_key(id: &str) -> String {
        format!("{}{id}", Self::MANUAL_PREFIX)
    }

    pub fn input(&self, name: &str) -> Option<&InputDeclaration> {
        self.inputs.iter().find(|input| input.name == name)
    }

    pub fn declares(&self, name: &str) -> bool {
        self.input(name).is_some()
    }

    pub fn manual_run(&self, inputs: Vec<(String, Value)>) -> Automation {
        Automation {
            id: Self::manual_key(&self.id),
            title: self.title.clone(),
            enabled: true,
            tags: Vec::new(),
            cooldown_seconds: 0,
            trigger: Trigger {
                event: EventName::Manual,
                filters: Filters::default(),
            },
            run: RunSettings::default(),
            workflow: Some(WorkflowCall {
                id: self.id.clone(),
                inputs: inputs
                    .into_iter()
                    .map(|(name, value)| (name, InputValue::Literal(value)))
                    .collect(),
            }),
        }
    }
}
