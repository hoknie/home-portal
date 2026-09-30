import type { Trace } from "@/entities/automation/@x/workflow";

import type { InputDeclaration, Step, Workflow, WorkflowCatalogue } from "../schema";
import { scopeAt } from "../scope";
import { type Path, everyStep } from "../tree";
import { itemSample } from "../transforming/known";
import { type PortalValues, portalSuggestions } from "../portal";
import { jsonKeys } from "./json-keys";
import { lastOutput } from "./last-output";

export const SUGGESTION_GROUPS = ["inputs", "vars", "steps", "loop", "item", "event", "secrets", "portal"] as const;

export type SuggestionGroup = (typeof SUGGESTION_GROUPS)[number];

export type Suggestion = {
  value: string;
  group: SuggestionGroup;
  description: { key: string; params: Record<string, string | number> };
  example?: string;
  warning?: string;
};

export type SuggestionContext = {
  steps: Step[];
  inputs: (string | InputDeclaration)[];
  path: Path;
  field: string;
  catalogue: WorkflowCatalogue | undefined;
  workflows: Workflow[];
  eventFields: { name: string; sample: string }[];
  secrets: { name: string; set: boolean }[];
  lastRun: Trace | null;
  portal?: PortalValues | null;
};

function variablesOf(workflow: Workflow | undefined): string[] {
  const found = new Set<string>();
  everyStep(workflow?.steps ?? [], (step) => {
    if (step.kind === "set" && step.variable) {
      found.add(step.variable);
    }
  });
  return [...found];
}


function stepSuggestions(context: SuggestionContext, step: { id: string; kind: string }, called: Step | undefined): Suggestion[] {
  const results = context.catalogue?.kinds.find((kind) => kind.name === step.kind)?.results ?? [];
  const own = results.map<Suggestion>((result) => ({
    value: `steps.${step.id}.${result}`,
    group: "steps",
    description: { key: `results.${step.kind}.${result}`, params: { step: step.id } },
  }));
  if (step.kind === "http") {
    own.push(
      ...jsonKeys(called?.response_sample ?? lastOutput(context.lastRun, step.id)).map<Suggestion>((key) => ({
        value: `steps.${step.id}.json.${key.path}`,
        group: "steps",
        description: { key: called?.response_sample ? "suggestions.sampleKey" : "suggestions.jsonKey", params: { step: step.id } },
        example: key.example,
      })),
    );
  }
  if (step.kind === "workflow") {
    const workflow = context.workflows.find((candidate) => candidate.id === called?.workflow);
    own.push(
      ...variablesOf(workflow).map<Suggestion>((name) => ({
        value: `steps.${step.id}.vars.${name}`,
        group: "steps",
        description: { key: "suggestions.calledVariable", params: { step: step.id, name, workflow: workflow?.title ?? "" } },
      })),
    );
  }
  return own;
}

function itemSuggestions(context: SuggestionContext): Suggestion[] {
  const sample = itemSample(context);
  return [
    { value: "item", group: "item", description: { key: "suggestions.item.value", params: {} } },
    { value: "index", group: "item", description: { key: "suggestions.item.index", params: {} } },
    ...jsonKeys(sample ? JSON.stringify(sample.value) : null).map<Suggestion>((key) => ({
      value: `item.${key.path}`,
      group: "item",
      description: { key: "suggestions.item.key", params: { key: key.path } },
      example: key.example,
    })),
  ];
}

export function suggestionsAt(context: SuggestionContext): Suggestion[] {
  const scope = scopeAt(context.steps, context.path, context.inputs, context.field);
  const byId = new Map<string, Step>();
  everyStep(context.steps, (step) => byId.set(step.id, step));
  return [
    ...scope.inputs.map<Suggestion>((name) => ({ value: `inputs.${name}`, group: "inputs", description: { key: "suggestions.input", params: { name } } })),
    ...scope.vars.flatMap<Suggestion>((name) => {
      const setter = byId.get(scope.setBy[name] ?? "");
      return [
        { value: `vars.${name}`, group: "vars", description: { key: "suggestions.variable", params: { name, step: scope.setBy[name] ?? "" } } },
        ...Object.keys(setter?.object ?? {})
          .filter((key) => /^[A-Za-z0-9_-]+$/.test(key))
          .map<Suggestion>((key) => ({
            value: `vars.${name}.${key}`,
            group: "vars",
            description: { key: "suggestions.variableKey", params: { name, key } },
          })),
      ];
    }),
    ...scope.steps.flatMap((step) => stepSuggestions(context, step, byId.get(step.id))),
    ...(scope.inLoop
      ? (["item", "index"] as const).map<Suggestion>((name) => ({ value: `loop.${name}`, group: "loop", description: { key: `suggestions.loop.${name}`, params: {} } }))
      : []),
    ...(scope.inTransform ? itemSuggestions(context) : []),
    ...context.eventFields.map<Suggestion>((field) => ({
      value: `event.${field.name}`,
      group: "event",
      description: { key: "suggestions.event", params: { name: field.name } },
      example: field.sample,
    })),
    ...context.secrets.map<Suggestion>((secret) => ({
      value: `secrets.${secret.name}`,
      group: "secrets",
      description: { key: "suggestions.secret", params: { name: secret.name } },
      warning: secret.set ? undefined : "suggestions.secretUnset",
    })),
    ...portalSuggestions(context.portal).map<Suggestion>((entry) => ({
      value: entry.value,
      group: "portal",
      description: { key: `suggestions.portal.${entry.kind}`, params: { subject: entry.subject } },
      example: entry.example,
    })),
  ];
}
