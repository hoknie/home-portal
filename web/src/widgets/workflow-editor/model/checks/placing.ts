import { type Path, ROOT, type Step, type Target, insideLoop, needsLoop, parsePath, pathText } from "@/entities/workflow";

import type { Problems } from "./validation";

const ENTRY = /^workflows\[\d+\]\./;

export function problemsFromServer(errors: { field: string; message: string }[]): Problems {
  const problems: Problems = {};
  for (const error of errors) {
    problems[error.field.replace(ENTRY, "")] = error.message;
  }
  return problems;
}

export function blocksToOpen(problems: Problems): string[] {
  const open = new Set<string>();
  for (const key of Object.keys(problems)) {
    const { path } = parsePath(key);
    for (let depth = 1; depth <= path.length; depth += 1) {
      open.add(pathText(path.slice(0, depth) as Path));
    }
  }
  return [...open];
}

export function fieldKey(path: Path, field: string) {
  return `${pathText(path)}.${field}`;
}

export type SlotContext = { inBranch: boolean; inLoop: boolean };

export function slotContext(target: Target): SlotContext {
  return { inBranch: target.list !== ROOT, inLoop: insideLoop(target) };
}

export function placeable(step: Step, target: Target) {
  return !needsLoop(step) || insideLoop(target);
}

export function kindPlaceable(kind: string, target: Target) {
  return placeable({ id: kind, kind }, target);
}
