import { type Condition, type KindField, type Step, type WorkflowCatalogue, everyStep, fitsType, pathText, variableProblem } from "@/entities/workflow";

import type { Draft } from "../draft";

export type Problems = Record<string, string>;

export const WORKFLOW_ID = /^[a-z][a-z0-9-]{0,62}$/;
export const STEP_ID = /^[a-z][a-z0-9_]{0,62}$/;
export const INPUT_NAME = /^[a-z][a-z0-9_]{0,62}$/;
export const RESERVED_IDS = ["catalogue", "runs"];
export const LONGEST_TITLE = 120;
export const LONGEST_TIMEOUT = 3600;
export const SCHEMES = /^https?:\/\//i;

const LOOP_MODES = ["repeat", "for_each", "while"] as const;

function valueOf(step: Step, field: KindField): unknown {
  return (step as Record<string, unknown>)[field.name];
}

function missing(value: unknown) {
  return value === undefined || value === null || (typeof value === "string" && value.trim() === "");
}

function conditionProblems(condition: Condition, at: string, problems: Problems) {
  const rows = condition.all ?? condition.any;
  if (rows !== undefined) {
    const key = condition.all !== undefined ? "all" : "any";
    rows.forEach((row, index) => conditionProblems(row, `${at}.${key}[${index}]`, problems));
    return;
  }
  if (missing(condition.left)) {
    problems[`${at}.left`] = "validation.required";
  }
}

function fieldProblems(step: Step, field: KindField, at: string, problems: Problems) {
  const value = valueOf(step, field);
  if (field.required && missing(value) && field.type !== "steps") {
    problems[`${at}.${field.name}`] = "validation.required";
    return;
  }
  if (field.type === "integer" && typeof value === "string" && !(field.templated && value.includes("{{"))) {
    problems[`${at}.${field.name}`] = "workflowEditor.problems.numberOrTemplate";
  }
  if (field.type === "integer" && typeof value === "number") {
    const low = field.minimum ?? -Infinity;
    const high = field.maximum ?? Infinity;
    if (!Number.isInteger(value) || value < low || value > high) {
      problems[`${at}.${field.name}`] = "workflowEditor.problems.between";
    }
  }
  if (field.type === "condition" && value !== undefined) {
    conditionProblems(value as Condition, `${at}.${field.name}`, problems);
  }
  if (field.type === "branches" && Array.isArray(value) && (value.length < (field.minimum ?? 2) || value.length > (field.maximum ?? 4))) {
    problems[`${at}.${field.name}`] = "workflowEditor.problems.branches";
  }
}

function stepProblems(step: Step, at: string, catalogue: WorkflowCatalogue, problems: Problems) {
  if (!STEP_ID.test(step.id)) {
    problems[`${at}.id`] = "workflowEditor.problems.stepId";
  }
  const kind = catalogue.kinds.find((entry) => entry.name === step.kind);
  if (kind === undefined) {
    problems[`${at}.kind`] = "workflowEditor.problems.kind";
    return;
  }
  for (const field of kind.fields) {
    fieldProblems(step, field, at, problems);
  }
  if (step.kind === "loop" && LOOP_MODES.filter((mode) => step[mode] !== undefined && step[mode] !== "").length !== 1) {
    problems[`${at}.repeat`] = "workflowEditor.problems.loopMode";
  }
  for (const name of Object.keys(step.kind === "script" ? (step.env ?? {}) : {})) {
    const problem = variableProblem(name);
    if (problem) {
      problems[`${at}.env.${name}`] = `workflowEditor.problems.${problem}`;
    }
  }
  if (step.kind === "http" && step.url && !step.url.includes("{{") && !SCHEMES.test(step.url)) {
    problems[`${at}.url`] = "workflowEditor.problems.scheme";
  }
}

export function problemsOf(draft: Draft, catalogue: WorkflowCatalogue, taken: string[]): Problems {
  const problems: Problems = {};
  if (draft.title.trim() === "" || draft.title.length > LONGEST_TITLE) {
    problems.title = "validation.automationTitle";
  }
  if (!WORKFLOW_ID.test(draft.id)) {
    problems.id = "validation.automationId";
  } else if (RESERVED_IDS.includes(draft.id) || taken.includes(draft.id)) {
    problems.id = "workflowEditor.problems.takenId";
  }
  if (!Number.isInteger(draft.timeout_seconds) || draft.timeout_seconds < 1 || draft.timeout_seconds > LONGEST_TIMEOUT) {
    problems.timeout_seconds = "validation.automationTimeout";
  }
  draft.inputs.forEach((input, index) => {
    if (!INPUT_NAME.test(input.name.trim()) || draft.inputs.findIndex((other) => other.name.trim() === input.name.trim()) !== index) {
      problems[`inputs[${index}]`] = "workflowEditor.problems.input";
    } else if (input.default !== null && !fitsType(input.type, input.default)) {
      problems[`inputs[${index}].default`] = "workflowEditor.problems.inputDefault";
    }
  });
  if (draft.steps.length === 0) {
    problems.steps = "workflowEditor.problems.noSteps";
  }
  const seen = new Set<string>();
  everyStep(draft.steps, (step, path) => {
    const at = pathText(path);
    stepProblems(step, at, catalogue, problems);
    if (seen.has(step.id)) {
      problems[`${at}.id`] = "workflowEditor.problems.duplicateId";
    }
    seen.add(step.id);
  });
  return problems;
}

export function problemsAt(problems: Problems, at: string) {
  return Object.entries(problems).filter(([key]) => key === at || key.startsWith(`${at}.`));
}
