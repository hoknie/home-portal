import { type TemplateName, templateNames } from "./check";

export const HEADER_NAMES = ["Authorization", "Content-Type", "Accept", "User-Agent", "X-Api-Key"] as const;

export const HTTP_STATUSES = ["200", "201", "204", "400", "401", "403", "404", "500", "503"] as const;

export const OUTCOME_VALUES = ["succeeded", "failed"] as const;

export const BRANCH_VALUES = ["then", "else"] as const;

export type ValueSuggestion = { value: string; label: string; detail?: string; disabled?: boolean };

function resultOf(name: TemplateName | undefined): string | null {
  if (!name?.valid) {
    return null;
  }
  const parts = name.name.split(".");
  return parts[0] === "steps" && parts.length === 3 ? parts[2] : null;
}

export function knownValues(left: string, states: string[]): ValueSuggestion[] {
  const names = templateNames(left.trim());
  const result = names.length === 1 && names[0].start === 0 && names[0].end === left.trim().length ? resultOf(names[0]) : null;
  const plain = (values: readonly string[]) => values.map((value) => ({ value, label: value }));
  switch (result) {
    case "state":
      return plain(states);
    case "status":
      return plain(HTTP_STATUSES);
    case "branch":
      return plain(BRANCH_VALUES);
    case "outcome":
      return plain(OUTCOME_VALUES);
    default:
      return [];
  }
}

export function serviceValues(services: { id: string; name: string }[]): ValueSuggestion[] {
  return services.map((service) => ({ value: service.id, label: service.name, detail: service.id }));
}

export function scriptValues(scripts: { path: string; runnable: boolean; problem: string | null }[]): ValueSuggestion[] {
  return scripts.map((script) => ({ value: script.path, label: script.path, detail: script.problem ?? undefined, disabled: !script.runnable }));
}

export function workflowValues(workflows: { id: string; title: string; inputs: { name: string }[] }[]): ValueSuggestion[] {
  return workflows.map((workflow) => ({ value: workflow.id, label: workflow.title, detail: workflow.inputs.map((input) => input.name).join(", ") }));
}

export function variableValues(vars: string[]): ValueSuggestion[] {
  return vars.map((name) => ({ value: name, label: name }));
}

export function headerValues(): ValueSuggestion[] {
  return HEADER_NAMES.map((name) => ({ value: name, label: name }));
}
