use std::collections::{BTreeMap, BTreeSet};

use portal_feature::FieldError;

use super::filter_checks::chain_problem;
use super::templates::templates_of;
use crate::services::workflow::evaluating::children_of;
use crate::services::workflow::evaluating::placeholders_in;
use crate::types::{Step, StepKind, ValueType, Workflow, result_type};

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub inputs: BTreeSet<String>,
    pub vars: BTreeSet<String>,
    pub steps: BTreeSet<String>,
    pub kinds: BTreeMap<String, &'static str>,
    pub input_types: BTreeMap<String, ValueType>,
    pub loops: usize,
    pub items: usize,
}

impl Scope {
    pub fn of(workflow: &Workflow) -> Scope {
        Scope {
            inputs: workflow
                .inputs
                .iter()
                .map(|input| input.name.clone())
                .collect(),
            input_types: workflow
                .inputs
                .iter()
                .map(|input| (input.name.clone(), input.input_type.value_type()))
                .collect(),
            ..Scope::default()
        }
    }

    pub fn errors(workflow: &Workflow) -> Vec<FieldError> {
        let mut scope = Scope::of(workflow);
        let mut errors = Vec::new();
        scope.check_steps(&workflow.steps, "steps", &mut errors);
        errors
    }

    pub fn allows(&self, name: &str) -> Result<(), String> {
        let mut parts = name.split('.');
        let namespace = parts.next().unwrap_or_default();
        let first = parts.next().unwrap_or_default();
        let fine = match namespace {
            "event" | "secrets" => true,
            "inputs" => self.inputs.contains(first),
            "vars" => self.vars.contains(first),
            "steps" => self.steps.contains(first),
            "loop" => self.loops > 0 && matches!(first, "item" | "index"),
            "item" | "index" => self.items > 0,
            _ => false,
        };
        if fine {
            return Ok(());
        }
        Err(match namespace {
            "inputs" => {
                format!("names {{{{{name}}}}}, but {first} is not an input of this workflow")
            }
            "vars" => {
                format!("names {{{{{name}}}}}, but no earlier step sets the variable {first}")
            }
            "steps" => format!("names {{{{{name}}}}}, but no step {first} comes earlier"),
            "loop" => format!(
                "names {{{{{name}}}}}, which exists only inside a loop as loop.item and loop.index"
            ),
            "item" | "index" => format!(
                "names {{{{{name}}}}}, which exists only inside a transform's filter and map"
            ),
            _ => format!("names {{{{{name}}}}}, which is not a value a workflow knows"),
        })
    }

    fn check_steps(&mut self, steps: &[Step], path: &str, errors: &mut Vec<FieldError>) {
        for (index, step) in steps.iter().enumerate() {
            let here = format!("{path}[{index}]");
            let inside_loop = matches!(step.kind, StepKind::Loop { .. });
            for (field, template) in templates_of(step) {
                let own_loop = inside_loop && field.starts_with("while");
                let own_items = field.starts_with("operations");
                self.loops += usize::from(own_loop);
                self.items += usize::from(own_items);
                for placeholder in placeholders_in(template) {
                    let problem = self.allows(placeholder.name).err().or_else(|| {
                        match &placeholder.filters {
                            Ok(filters) => chain_problem(self.type_of(placeholder.name), filters),
                            Err(message) => Some(message.clone()),
                        }
                        .map(|problem| format!("{{{{{}}}}}: {problem}", placeholder.name))
                    });
                    if let Some(message) = problem {
                        errors.push(FieldError::new(format!("{here}.{field}"), message));
                    }
                }
                self.loops -= usize::from(own_loop);
                self.items -= usize::from(own_items);
            }
            self.steps.insert(step.id.clone());
            self.kinds.insert(step.id.clone(), step.kind.name());
            if inside_loop {
                self.loops += 1;
            }
            for (list, children) in children_of(step) {
                self.check_steps(children, &format!("{here}.{list}"), errors);
            }
            if inside_loop {
                self.loops -= 1;
            }
            if let StepKind::Set { variable, .. } = &step.kind {
                self.vars.insert(variable.clone());
            }
        }
    }

    pub fn type_of(&self, name: &str) -> ValueType {
        let parts: Vec<&str> = name.split('.').collect();
        match parts.as_slice() {
            ["inputs", name] => self
                .input_types
                .get(*name)
                .copied()
                .unwrap_or(ValueType::Text),
            ["secrets", _] => ValueType::Text,
            ["event", ..] => ValueType::Text,
            ["loop", "index"] | ["index"] => ValueType::Number,
            ["steps", id, field] => self
                .kinds
                .get(*id)
                .map_or(ValueType::Any, |kind| result_type(kind, field)),
            _ => ValueType::Any,
        }
    }
}
