use crate::types::{Step, StepKind};

pub fn every_step(steps: &[Step], path: &str, visit: &mut dyn FnMut(&Step, &str)) {
    for (index, step) in steps.iter().enumerate() {
        let here = format!("{path}[{index}]");
        visit(step, &here);
        for (list, children) in children_of(step) {
            every_step(children, &format!("{here}.{list}"), visit);
        }
    }
}

pub fn children_of(step: &Step) -> Vec<(String, &[Step])> {
    match &step.kind {
        StepKind::If {
            then, otherwise, ..
        } => vec![
            ("then".to_string(), then.as_slice()),
            ("else".to_string(), otherwise.as_slice()),
        ],
        StepKind::Loop { body, .. } => vec![("body".to_string(), body.as_slice())],
        StepKind::Parallel { branches } => branches
            .iter()
            .enumerate()
            .map(|(index, branch)| (format!("branches[{index}]"), branch.as_slice()))
            .collect(),
        _ => Vec::new(),
    }
}
