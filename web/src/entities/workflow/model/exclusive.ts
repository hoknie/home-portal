import { emptyRow } from "./conditions";
import type { Kind, Step } from "./schema";

const STARTING: Record<string, () => unknown> = {
  repeat: () => 1,
  for_each: () => "",
  while: () => emptyRow(),
  value: () => "",
  json: () => "",
  list: () => [""],
  object: () => ({ "": "" }),
};

export function startingValue(field: string): unknown {
  return (STARTING[field] ?? (() => ""))();
}

export function chosenOf(step: Step, group: string[]): string {
  return group.find((field) => (step as Record<string, unknown>)[field] !== undefined) ?? group[0];
}

export function withChoice(step: Step, group: string[], field: string): Step {
  const copy: Record<string, unknown> = { ...step };
  for (const other of group) {
    delete copy[other];
  }
  copy[field] = startingValue(field);
  return copy as Step;
}

export function hiddenFields(step: Step, kind: Kind | undefined): Set<string> {
  const hidden = new Set<string>();
  for (const group of kind?.exclusive ?? []) {
    const chosen = chosenOf(step, group);
    group.filter((field) => field !== chosen).forEach((field) => hidden.add(field));
  }
  return hidden;
}
