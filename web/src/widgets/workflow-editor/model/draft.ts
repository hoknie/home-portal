import type { Step, Workflow, WorkflowRequest } from "@/entities/workflow";

export type Draft = WorkflowRequest;

export const DEFAULT_TIMEOUT = 300;

export const emptyDraft: Draft = {
  id: "",
  title: "",
  enabled: true,
  description: null,
  tags: [],
  timeout_seconds: DEFAULT_TIMEOUT,
  inputs: [],
  steps: [],
};

export function draftOf(workflow: Workflow | null): Draft {
  if (workflow === null) {
    return emptyDraft;
  }
  return {
    id: workflow.id,
    title: workflow.title,
    enabled: workflow.enabled,
    description: workflow.description,
    tags: workflow.tags,
    timeout_seconds: workflow.timeout_seconds,
    inputs: workflow.inputs,
    steps: workflow.steps,
  };
}

function withoutEmpty(step: Step): Step {
  const copy: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(step)) {
    if (value === undefined || (typeof value === "string" && value === "" && key !== "id" && key !== "kind")) {
      continue;
    }
    if (Array.isArray(value) && (key === "then" || key === "else" || key === "body")) {
      copy[key] = value.map((child) => withoutEmpty(child as Step));
    } else if (key === "branches" && Array.isArray(value)) {
      copy[key] = (value as Step[][]).map((branch) => branch.map(withoutEmpty));
    } else {
      copy[key] = value;
    }
  }
  return copy as Step;
}

export function requestOf(draft: Draft): WorkflowRequest {
  return {
    ...draft,
    description: draft.description?.trim() ? draft.description.trim() : null,
    inputs: draft.inputs.map((input) => ({ ...input, name: input.name.trim(), description: input.description?.trim() ? input.description.trim() : null })),
    steps: draft.steps.map(withoutEmpty),
  };
}

export function sameDraft(left: Draft, right: Draft) {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function sameSteps(left: Draft, right: Draft) {
  return JSON.stringify(left.steps) === JSON.stringify(right.steps);
}
