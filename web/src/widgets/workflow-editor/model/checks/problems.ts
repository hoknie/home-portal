import {
  type PortalValues,
  type Step,
  type Workflow,
  type WorkflowCatalogue,
  checkTemplate,
  everyStep,
  flowOrder,
  jsonKeys,
  namedInputs,
  parsePath,
  pathText,
  scopeAt,
  templateNames,
  templatesOf,
  unreachableSteps,
} from "@/entities/workflow";
import type { Trace } from "@/entities/automation";

import type { Draft } from "../draft";
import { type Problems, problemsOf } from "./validation";

export type Severity = "error" | "warning";

export type Problem = { at: string; severity: Severity; key: string | null; params: Record<string, string | number>; text: string | null };

export type ProblemContext = {
  draft: Draft;
  catalogue: WorkflowCatalogue;
  taken: string[];
  server: Problems;
  secrets: { name: string; set: boolean }[];
  lastRun: Trace | null;
  portal?: PortalValues | null;
  workflow: Workflow | null;
};

function lastOutput(trace: Trace | null, step: string) {
  return (trace?.entries ?? []).filter((entry) => entry.step === step && entry.output).at(-1)?.output ?? null;
}

function templateProblems(context: ProblemContext): Problem[] {
  const found: Problem[] = [];
  const inputs = namedInputs(context.draft.inputs);
  const unset = new Set(context.secrets.filter((secret) => !secret.set).map((secret) => secret.name));
  const samples = new Map<string, string>();
  everyStep(context.draft.steps, (step) => {
    if (step.response_sample) {
      samples.set(step.id, step.response_sample);
    }
  });
  everyStep(context.draft.steps, (step: Step, path) => {
    for (const { field, text } of templatesOf(step)) {
      const at = `${pathText(path)}.${field}`;
      const scope = scopeAt(context.draft.steps, path, inputs, field);
      for (const problem of checkTemplate(text, scope, context.portal ?? null)) {
        found.push({ at, severity: "error", key: `workflowHelp.reasons.${problem.reason}`, params: problem.params, text: null });
      }
      for (const name of templateNames(text).filter((entry) => entry.valid).map((entry) => entry.name.split("."))) {
        if (name[0] === "secrets" && unset.has(name[1])) {
          found.push({ at, severity: "warning", key: "workflowEditor.problems.secretUnset", params: { name: name[1] }, text: null });
        }
        if (name[0] === "steps" && name[2] === "json" && name.length > 3) {
          const output = samples.get(name[1]) ?? lastOutput(context.lastRun, name[1]);
          const key = name.slice(3).join(".");
          if (output !== null && !jsonKeys(output).some((entry) => entry.path === key)) {
            found.push({ at, severity: "warning", key: samples.has(name[1]) ? "workflowEditor.problems.unknownSampleKey" : "workflowEditor.problems.unseenKey", params: { step: name[1], key }, text: null });
          }
        }
      }
    }
  });
  return found;
}

function rank(steps: Step[], at: string) {
  const { path } = parsePath(at);
  if (path.length === 0) {
    return -1;
  }
  const order = flowOrder(steps).map(pathText);
  const index = order.indexOf(pathText(path));
  return index < 0 ? order.length : index;
}

export function problemsFor(context: ProblemContext): Problem[] {
  const structural = Object.entries(problemsOf(context.draft, context.catalogue, context.taken)).map<Problem>(([at, key]) => ({ at, severity: "error", key, params: {}, text: null }));
  const server = Object.entries(context.server).map<Problem>(([at, text]) => ({ at, severity: "error", key: null, params: {}, text }));
  const unused =
    context.workflow !== null && context.workflow.used_by.length === 0
      ? [{ at: "", severity: "warning" as const, key: "workflowEditor.problems.notStarted", params: {}, text: null }]
      : [];
  const lost = unreachableSteps(context.draft.steps).map<Problem>((path) => ({
    at: pathText(path),
    severity: "warning",
    key: "workflowEditor.problems.neverRuns",
    params: {},
    text: null,
  }));
  const all = [...structural, ...templateProblems(context), ...server, ...lost, ...unused];
  return all
    .map((problem, index) => ({ problem, index, rank: rank(context.draft.steps, problem.at) }))
    .sort((left, right) => left.rank - right.rank || left.index - right.index)
    .map((entry) => entry.problem);
}

export function blocking(problems: Problem[]) {
  return problems.filter((problem) => problem.severity === "error");
}
