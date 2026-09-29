import type { Step } from "../schema";
import { type Path, ROOT, type Target, childList, listsOf } from "../tree";

export const LOOP_EXITS = ["break", "continue"] as const;

export type Closing = "succeeded" | "failed" | "break" | "continue";

export function closingOf(step: Step): Closing | null {
  if (step.kind === "stop") {
    return step.outcome === "failed" ? "failed" : "succeeded";
  }
  return step.kind === "break" || step.kind === "continue" ? step.kind : null;
}

export function closedAt(steps: Step[]): number {
  return steps.findIndex(closes);
}

export function listCloses(steps: Step[]) {
  return closedAt(steps) >= 0;
}

export function closes(step: Step): boolean {
  if (closingOf(step) !== null) {
    return true;
  }
  if (step.kind === "if" || step.kind === "parallel") {
    const lists = listsOf(step);
    return lists.length > 0 && lists.every((list) => listCloses(childList(step, list)));
  }
  return false;
}

export function unreachableSteps(steps: Step[], owner: Path = [], list = ROOT): Path[] {
  const found: Path[] = [];
  const closed = closedAt(steps);
  steps.forEach((step, index) => {
    const path = [...owner, { list, index }];
    if (closed >= 0 && index > closed) {
      found.push(path);
      return;
    }
    for (const child of listsOf(step)) {
      found.push(...unreachableSteps(childList(step, child), path, child));
    }
  });
  return found;
}

export function needsLoop(step: Step): boolean {
  if (step.kind === "break" || step.kind === "continue") {
    return true;
  }
  return step.kind === "if" && listsOf(step).some((list) => childList(step, list).some(needsLoop));
}

export function insideLoop(target: Target): boolean {
  const lists = [target.list, ...target.owner.map((place) => place.list).reverse()];
  for (const list of lists) {
    if (list === "body") {
      return true;
    }
    if (list !== "then" && list !== "else") {
      return false;
    }
  }
  return false;
}
