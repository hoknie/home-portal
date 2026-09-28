use super::every_step;
use crate::types::{Automation, StepKind, Webhook, WebhookAction, Workflow, WorkflowUsage};

pub fn users_of(
    id: &str,
    (automations, webhooks, workflows): (&[Automation], &[Webhook], &[Workflow]),
) -> Vec<WorkflowUsage> {
    let automations = automations
        .iter()
        .filter(|automation| {
            automation
                .workflow
                .as_ref()
                .is_some_and(|call| call.id == id)
        })
        .map(|automation| WorkflowUsage {
            kind: WorkflowUsage::AUTOMATION,
            id: automation.id.clone(),
            title: automation.title.clone(),
        });
    let webhooks = webhooks
        .iter()
        .filter(|webhook| matches!(&webhook.action, WebhookAction::Workflow(call) if call.id == id))
        .map(|webhook| WorkflowUsage {
            kind: WorkflowUsage::WEBHOOK,
            id: webhook.id.clone(),
            title: webhook.title.clone(),
        });
    let workflows = workflows
        .iter()
        .filter(|workflow| workflow.id != id && calls(workflow, id))
        .map(|workflow| WorkflowUsage {
            kind: WorkflowUsage::WORKFLOW,
            id: workflow.id.clone(),
            title: workflow.title.clone(),
        });
    automations.chain(webhooks).chain(workflows).collect()
}

fn calls(workflow: &Workflow, id: &str) -> bool {
    let mut found = false;
    every_step(&workflow.steps, "steps", &mut |step, _| {
        if matches!(&step.kind, StepKind::Call { workflow, .. } if workflow == id) {
            found = true;
        }
    });
    found
}
