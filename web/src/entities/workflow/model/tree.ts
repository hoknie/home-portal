import { slugOf } from "@/shared/lib/slug";

import type { Kind, Step } from "./schema";

export const ROOT = "steps";

export type Place = { list: string; index: number };

export type Path = Place[];

export type Target = { owner: Path; list: string; index: number };

const PLACE = /^(steps|then|else|body|branches\[\d+\])\[(\d+)\]/;

export function pathText(path: Path) {
  return path.map((place) => `${place.list}[${place.index}]`).join(".");
}

export function parsePath(text: string): { path: Path; field: string } {
  const path: Path = [];
  let rest = text;
  for (;;) {
    const match = PLACE.exec(rest);
    if (match === null) {
      break;
    }
    path.push({ list: match[1], index: Number(match[2]) });
    rest = rest.slice(match[0].length).replace(/^\./, "");
  }
  return { path, field: rest };
}

export function listsOf(step: Step): string[] {
  switch (step.kind) {
    case "if":
      return ["then", "else"];
    case "loop":
      return ["body"];
    case "parallel":
      return (step.branches ?? []).map((_, index) => `branches[${index}]`);
    default:
      return [];
  }
}

function branchIndex(list: string) {
  const match = /^branches\[(\d+)\]$/.exec(list);
  return match === null ? null : Number(match[1]);
}

export function childList(step: Step, list: string): Step[] {
  const branch = branchIndex(list);
  if (branch !== null) {
    return step.branches?.[branch] ?? [];
  }
  if (list === "then") {
    return step.then ?? [];
  }
  if (list === "else") {
    return step.else ?? [];
  }
  if (list === "body") {
    return Array.isArray(step.body) ? step.body : [];
  }
  return [];
}

export function withChildList(step: Step, list: string, children: Step[]): Step {
  const branch = branchIndex(list);
  if (branch !== null) {
    const branches = [...(step.branches ?? [])];
    branches[branch] = children;
    return { ...step, branches };
  }
  return { ...step, [list]: children };
}

export function at(steps: Step[], path: Path): Step | undefined {
  let list = steps;
  let found: Step | undefined;
  for (const [position, place] of path.entries()) {
    list = position === 0 ? steps : found === undefined ? [] : childList(found, place.list);
    found = list[place.index];
    if (found === undefined) {
      return undefined;
    }
  }
  return found;
}

export function stepsIn(steps: Step[], owner: Path, list: string): Step[] {
  if (owner.length === 0) {
    return steps;
  }
  const step = at(steps, owner);
  return step === undefined ? [] : childList(step, list);
}

function updateInStep(step: Step, rest: Path, list: string, change: (steps: Step[]) => Step[]): Step {
  if (rest.length === 0) {
    return withChildList(step, list, change(childList(step, list)));
  }
  const [next, ...more] = rest;
  const children = childList(step, next.list).map((child, index) => (index === next.index ? updateInStep(child, more, list, change) : child));
  return withChildList(step, next.list, children);
}

export function updateList(steps: Step[], owner: Path, list: string, change: (steps: Step[]) => Step[]): Step[] {
  if (owner.length === 0) {
    return change(steps);
  }
  const [first, ...rest] = owner;
  return steps.map((step, index) => (index === first.index ? updateInStep(step, rest, list, change) : step));
}

export function updateAt(steps: Step[], path: Path, change: (step: Step) => Step): Step[] {
  const last = path.at(-1);
  if (last === undefined) {
    return steps;
  }
  return updateList(steps, path.slice(0, -1), last.list, (list) => list.map((step, index) => (index === last.index ? change(step) : step)));
}

export function insert(steps: Step[], target: Target, step: Step): Step[] {
  return updateList(steps, target.owner, target.list, (list) => {
    const index = Math.max(0, Math.min(target.index, list.length));
    return [...list.slice(0, index), step, ...list.slice(index)];
  });
}

export function remove(steps: Step[], path: Path): Step[] {
  const last = path.at(-1);
  if (last === undefined) {
    return steps;
  }
  return updateList(steps, path.slice(0, -1), last.list, (list) => list.filter((_, index) => index !== last.index));
}

function samePlaces(left: Path, right: Path) {
  return left.length === right.length && left.every((place, index) => place.list === right[index].list && place.index === right[index].index);
}

export function contains(outer: Path, inner: Path) {
  return inner.length >= outer.length && samePlaces(outer, inner.slice(0, outer.length));
}

function afterRemoval(path: Path, removed: Path): Path {
  const depth = removed.length - 1;
  const last = removed[depth];
  if (path.length <= depth || !samePlaces(path.slice(0, depth), removed.slice(0, depth))) {
    return path;
  }
  const place = path[depth];
  if (place.list !== last.list || place.index <= last.index) {
    return path;
  }
  return path.map((item, index) => (index === depth ? { ...item, index: item.index - 1 } : item));
}

export function move(steps: Step[], from: Path, target: Target): Step[] {
  const moving = at(steps, from);
  if (moving === undefined || contains(from, target.owner)) {
    return steps;
  }
  const destination = afterRemoval([...target.owner, { list: target.list, index: target.index }], from);
  const place = destination[destination.length - 1];
  return insert(remove(steps, from), { owner: destination.slice(0, -1), list: place.list, index: place.index }, moving);
}

export function moveBy(steps: Step[], path: Path, offset: -1 | 1): Step[] {
  const last = path.at(-1);
  if (last === undefined) {
    return steps;
  }
  const owner = path.slice(0, -1);
  const size = stepsIn(steps, owner, last.list).length;
  const index = last.index + offset;
  if (index < 0 || index >= size) {
    return steps;
  }
  return updateList(steps, owner, last.list, (list) => {
    const copy = [...list];
    [copy[last.index], copy[index]] = [copy[index], copy[last.index]];
    return copy;
  });
}

export function everyStep(steps: Step[], visit: (step: Step, path: Path) => void, owner: Path = [], list = ROOT) {
  steps.forEach((step, index) => {
    const here = [...owner, { list, index }];
    visit(step, here);
    for (const child of listsOf(step)) {
      everyStep(childList(step, child), visit, here, child);
    }
  });
}

export function idsOf(steps: Step[]) {
  const ids = new Set<string>();
  everyStep(steps, (step) => ids.add(step.id));
  return ids;
}

export function idFor(label: string, taken: Set<string>) {
  const base = slugOf(label).replace(/-/g, "_") || "step";
  if (!taken.has(base)) {
    return base;
  }
  for (let number = 2; ; number += 1) {
    const candidate = `${base}_${number}`;
    if (!taken.has(candidate)) {
      return candidate;
    }
  }
}

function freshIds(step: Step, taken: Set<string>): Step {
  const id = idFor(step.id, taken);
  taken.add(id);
  let copy: Step = { ...step, id };
  for (const list of listsOf(step)) {
    copy = withChildList(copy, list, childList(step, list).map((child) => freshIds(child, taken)));
  }
  return copy;
}

export function duplicate(steps: Step[], path: Path): Step[] {
  const original = at(steps, path);
  const last = path.at(-1);
  if (original === undefined || last === undefined) {
    return steps;
  }
  const copy = freshIds(original, idsOf(steps));
  return insert(steps, { owner: path.slice(0, -1), list: last.list, index: last.index + 1 }, copy);
}

function initial(kind: Kind): Partial<Step> {
  const values: Record<string, unknown> = {};
  for (const field of kind.fields) {
    if (field.type === "steps" && field.required) {
      values[field.name] = [];
    } else if (field.type === "branches") {
      values[field.name] = [[], []];
    } else if (field.type === "condition" && field.required) {
      values[field.name] = { left: "", op: "==", right: "" };
    } else if (field.type === "choice" && (field.required || field.default !== null)) {
      values[field.name] = field.default ?? field.choices[0];
    } else if (field.required && (field.type === "template" || field.type === "name" || field.type === "workflow" || field.type === "script")) {
      values[field.name] = "";
    } else if (field.required && field.type === "integer") {
      values[field.name] = Number(field.default ?? field.minimum ?? 1);
    }
  }
  if (kind.name === "loop") {
    values.repeat = 1;
  }
  return values as Partial<Step>;
}

export function newStep(kind: Kind, taken: Set<string>): Step {
  return { ...initial(kind), id: idFor(kind.name, taken), kind: kind.name };
}

export const LEGACY_TELEGRAM = "telegram";

export function withNotifySteps(steps: Step[]): Step[] {
  return steps.map((step) => {
    let copy: Step = step.kind === LEGACY_TELEGRAM ? { ...step, kind: "notify", channel: step.channel ?? LEGACY_TELEGRAM } : step;
    for (const list of listsOf(copy).filter((name) => childList(copy, name).length > 0)) {
      copy = withChildList(copy, list, withNotifySteps(childList(copy, list)));
    }
    return copy;
  });
}
