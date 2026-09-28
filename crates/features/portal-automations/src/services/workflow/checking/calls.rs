use std::collections::{BTreeMap, HashSet};

use portal_feature::FieldError;

use crate::services::workflow::evaluating::every_step;
use crate::types::{Automation, Catalogue, RawAutomation, StepKind, Workflow};

pub struct CallEdge {
    pub target: String,
    pub path: String,
    pub inputs: Vec<String>,
}

pub fn call_errors(workflows: &[(usize, Workflow)]) -> Vec<FieldError> {
    let declared: BTreeMap<&str, &Workflow> = workflows
        .iter()
        .map(|(_, workflow)| (workflow.id.as_str(), workflow))
        .collect();
    let mut errors = Vec::new();
    let mut graph: BTreeMap<String, Vec<CallEdge>> = BTreeMap::new();
    for (index, workflow) in workflows {
        let edges = edges_of(workflow, &format!("{}[{index}].steps", Workflow::SECTION));
        for edge in &edges {
            match declared.get(edge.target.as_str()) {
                None => errors.push(FieldError::new(
                    format!("{}.workflow", edge.path),
                    format!("names {:?}, which is not a workflow", edge.target),
                )),
                Some(target) => {
                    for input in &edge.inputs {
                        if !target.declares(input) {
                            errors.push(FieldError::new(
                                format!("{}.inputs.{input}", edge.path),
                                format!("is not an input of {}", target.id),
                            ));
                        }
                    }
                }
            }
        }
        graph.insert(workflow.id.clone(), edges);
    }
    let cycles = cycle_errors(&graph);
    if cycles.is_empty() {
        errors.extend(depth_errors(&graph));
    }
    errors.extend(cycles);
    errors
}

fn edges_of(workflow: &Workflow, root: &str) -> Vec<CallEdge> {
    let mut edges = Vec::new();
    every_step(&workflow.steps, root, &mut |step, path| {
        if let StepKind::Call { workflow, inputs } = &step.kind {
            edges.push(CallEdge {
                target: workflow.clone(),
                path: path.to_string(),
                inputs: inputs.iter().map(|(name, _)| name.clone()).collect(),
            });
        }
    });
    edges
}

fn cycle_errors(graph: &BTreeMap<String, Vec<CallEdge>>) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let mut reported = HashSet::new();
    for start in graph.keys() {
        let mut trail = vec![start.clone()];
        walk_cycles(graph, &mut trail, &mut |edge, cycle| {
            if reported.insert(edge.path.clone()) {
                errors.push(FieldError::new(
                    format!("{}.workflow", edge.path),
                    format!("closes a cycle of calls: {}", cycle.join(" → ")),
                ));
            }
        });
    }
    errors
}

fn walk_cycles(
    graph: &BTreeMap<String, Vec<CallEdge>>,
    trail: &mut Vec<String>,
    found: &mut dyn FnMut(&CallEdge, &[String]),
) {
    let Some(current) = trail.last().cloned() else {
        return;
    };
    for edge in graph.get(&current).into_iter().flatten() {
        if let Some(start) = trail.iter().position(|seen| *seen == edge.target) {
            let mut cycle = trail[start..].to_vec();
            cycle.push(edge.target.clone());
            found(edge, &cycle);
            continue;
        }
        if graph.contains_key(&edge.target) {
            trail.push(edge.target.clone());
            walk_cycles(graph, trail, found);
            trail.pop();
        }
    }
}

fn depth_errors(graph: &BTreeMap<String, Vec<CallEdge>>) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for edges in graph.values() {
        for edge in edges {
            if height(graph, &edge.target) + 1 > Workflow::DEEPEST_CALLS {
                errors.push(FieldError::new(
                    format!("{}.workflow", edge.path),
                    format!(
                        "calls nest too deep; workflows call each other at most {} deep",
                        Workflow::DEEPEST_CALLS
                    ),
                ));
            }
        }
    }
    errors
}

fn height(graph: &BTreeMap<String, Vec<CallEdge>>, id: &str) -> usize {
    graph
        .get(id)
        .into_iter()
        .flatten()
        .map(|edge| height(graph, &edge.target) + 1)
        .max()
        .unwrap_or(0)
}

pub fn automation_errors(
    workflows: &[(usize, Workflow)],
    automations: &[RawAutomation],
) -> Vec<FieldError> {
    let known: Vec<Automation> = automations
        .iter()
        .filter_map(|raw| Automation::decode(raw).ok())
        .collect();
    let mut errors = Vec::new();
    for (index, workflow) in workflows {
        let root = format!("{}[{index}].steps", Workflow::SECTION);
        every_step(&workflow.steps, &root, &mut |step, path| {
            let StepKind::Automation {
                automation, fields, ..
            } = &step.kind
            else {
                return;
            };
            let Some(target) = known.iter().find(|candidate| candidate.id == *automation) else {
                errors.push(FieldError::new(
                    format!("{path}.automation"),
                    format!("names {automation:?}, which is not an automation"),
                ));
                return;
            };
            let offered = Catalogue::fields_of(target.trigger.event);
            for (name, _) in fields {
                if !offered.contains(&name.as_str()) {
                    errors.push(FieldError::new(
                        format!("{path}.fields.{name}"),
                        format!(
                            "is not a field of {}; its fields are {}",
                            target.trigger.event.name(),
                            offered.join(", ")
                        ),
                    ));
                }
            }
        });
    }
    errors
}
