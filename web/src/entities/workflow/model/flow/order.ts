import type { Step } from "../schema";
import { type Path, at, childList, everyStep, listsOf, pathText } from "../tree";

export type Direction = "previous" | "next" | "into" | "out";

export function flowOrder(steps: Step[]): Path[] {
  const order: Path[] = [];
  everyStep(steps, (_, path) => order.push(path));
  return order;
}

export function neighbour(steps: Step[], current: Path | null, direction: Direction): Path | null {
  const order = flowOrder(steps);
  if (order.length === 0) {
    return null;
  }
  if (current === null) {
    return order[0];
  }
  const index = order.findIndex((path) => pathText(path) === pathText(current));
  if (direction === "previous") {
    return index > 0 ? order[index - 1] : order[0];
  }
  if (direction === "next") {
    return index >= 0 && index < order.length - 1 ? order[index + 1] : current;
  }
  if (direction === "out") {
    return current.length > 1 ? current.slice(0, -1) : current;
  }
  const step = at(steps, current);
  const list = step ? listsOf(step).find((name) => childList(step, name).length > 0) : undefined;
  return list ? [...current, { list, index: 0 }] : current;
}
