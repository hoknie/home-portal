import type { Trace } from "@/entities/automation/@x/workflow";

import type { FilterDescription, InputDeclaration, Step, WorkflowCatalogue } from "../schema";
import { scopeAt } from "../scope";
import { lastOutput } from "../suggestions/last-output";
import { type Path, at, everyStep } from "../tree";
import { applyFilter } from "./filters";
import { previewOperations } from "./operations";
import { type FilterCall, parseChain } from "./parse";
import { placeholderAt } from "./render";
import { certainType, givesAfter } from "./types";
import { type ValueType, textOf, typeOfValue, walk } from "./values";

export type KnownContext = { steps: Step[]; inputs: (string | InputDeclaration)[]; path: Path; field: string; lastRun: Trace | null; catalogue: WorkflowCatalogue | undefined };

export type Sample = { value: unknown; from: "sample" | "run" };

export type FilterOffer = { name: string; insert: string; label: string; example?: string; element: boolean };

const DEEPEST_SAMPLE = 4;
const LONGEST_EXAMPLE = 40;

function parsed(text: string | null | undefined): unknown {
  if (!text) {
    return undefined;
  }
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return undefined;
  }
}

function stepNamed(steps: Step[], id: string) {
  let found: Step | undefined;
  everyStep(steps, (step) => {
    found = step.id === id ? step : found;
  });
  return found;
}

export function inputSample(step: Step, context: KnownContext, depth = 0): Sample | null {
  const trimmed = (step.input ?? "").trim();
  const placeholder = trimmed.startsWith("{{") ? placeholderAt(trimmed.slice(2)) : null;
  if (placeholder === null || placeholder.filters === null || trimmed.length !== placeholder.length + 4) {
    return null;
  }
  const source = sampleOf(placeholder.name, context, depth + 1);
  if (source === null) {
    return null;
  }
  try {
    return { value: placeholder.filters.reduce<unknown>((value, call) => applyFilter(value, call), source.value), from: source.from };
  } catch {
    return null;
  }
}

export function sampleOf(name: string, context: KnownContext, depth = 0): Sample | null {
  const [namespace, id = "", field = "", ...path] = name.split(".");
  if (namespace !== "steps" || depth > DEEPEST_SAMPLE) {
    return null;
  }
  const step = stepNamed(context.steps, id);
  if (step?.kind === "http" && field === "json") {
    const own = parsed(step.response_sample);
    const run = parsed(lastOutput(context.lastRun, id));
    const value = own !== undefined ? own : run;
    return value === undefined ? null : { value: walk(value, path), from: own !== undefined ? "sample" : "run" };
  }
  if (step?.kind === "transform" && field === "value") {
    const input = inputSample(step, context, depth);
    if (input !== null) {
      const last = previewOperations(input.value, step.operations ?? []).at(-1);
      return last && last.error === null ? { value: walk(last.value, path), from: input.from } : null;
    }
    const snapshots = parsed(lastOutput(context.lastRun, id));
    return Array.isArray(snapshots) && snapshots.length > 0 ? { value: walk(snapshots.at(-1), path), from: "run" } : null;
  }
  return null;
}

export function operationIndex(field: string): number | null {
  const match = /^operations\[(\d+)\]/.exec(field);
  return match ? Number(match[1]) : null;
}

export function valueBefore(context: KnownContext, index: number): Sample | null {
  const step = at(context.steps, context.path);
  if (step?.kind !== "transform") {
    return null;
  }
  const input = inputSample(step, context);
  if (input === null) {
    return null;
  }
  if (index === 0) {
    return input;
  }
  const preview = previewOperations(input.value, (step.operations ?? []).slice(0, index)).at(-1);
  return preview && preview.error === null ? { value: preview.value, from: input.from } : null;
}

export function itemSample(context: KnownContext): Sample | null {
  const index = operationIndex(context.field);
  const before = index === null ? null : valueBefore(context, index);
  return before && Array.isArray(before.value) && before.value.length > 0 ? { value: before.value[0], from: before.from } : null;
}

export function knownType(name: string, context: KnownContext): ValueType {
  const scope = scopeAt(context.steps, context.path, context.inputs, context.field);
  const certain = certainType(name, scope);
  if (certain !== "any") {
    return certain;
  }
  const [namespace, ...path] = name.split(".");
  const setter = namespace === "vars" && path.length === 1 ? stepNamed(context.steps, scope.setBy[path[0]] ?? "") : undefined;
  if (setter?.list !== undefined) {
    return "list";
  }
  if (setter?.object !== undefined) {
    return "object";
  }
  const sample = namespace === "item" ? itemSample(context) : sampleOf(name, context);
  const value = namespace === "item" && sample ? walk(sample.value, path) : sample?.value;
  return sample === null || value === null || value === undefined ? "any" : typeOfValue(value);
}

function placeholderOf(argument: FilterDescription["arguments"][number]) {
  return argument.type === "number" ? "0" : argument.type === "text" ? JSON.stringify(argument.name) : '""';
}

function exampleOf(value: unknown, call: FilterCall): string | undefined {
  try {
    const text = textOf(applyFilter(value, call));
    return text.length > LONGEST_EXAMPLE ? `${text.slice(0, LONGEST_EXAMPLE - 1)}…` : text;
  } catch {
    return undefined;
  }
}

export function filterOffers(context: KnownContext, subject: string, chain: string): FilterOffer[] {
  const before = chain === "" ? [] : (parseChain(chain).filters ?? []);
  const type = givesAfter(knownType(subject, context), before);
  const sample = subject.startsWith("item") ? itemSample(context) : sampleOf(subject, context);
  let value: unknown;
  try {
    value = sample === null ? undefined : before.reduce<unknown>((current, call) => applyFilter(current, call), subject.startsWith("item") ? walk(sample.value, subject.split(".").slice(1)) : sample.value);
  } catch {
    value = undefined;
  }
  const filters = context.catalogue?.filters ?? [];
  const exact = filters.filter((filter) => type === "any" || filter.accepts.includes(type));
  const general = type === "any" ? [] : filters.filter((filter) => !exact.includes(filter) && filter.accepts.includes("any"));
  const first = Array.isArray(value) && value.length > 0 ? typeOfValue(value[0]) : null;
  const elementwise =
    type === "list"
      ? filters.filter((filter) => filter.element && !exact.includes(filter) && (first === null || filter.accepts.includes(first as never)))
      : [];
  return [...exact, ...elementwise, ...general].map((filter) => {
    const required = filter.arguments.filter((argument) => argument.required);
    const insert = required.length === 0 ? filter.name : `${filter.name}(${required.map(placeholderOf).join(", ")})`;
    const call = { name: filter.name, arguments: required.map((argument) => JSON.parse(placeholderOf(argument)) as unknown) };
    return {
      name: filter.name,
      insert,
      label: filter.arguments.length === 0 ? filter.name : `${filter.name}(${filter.arguments.map((argument) => argument.name).join(", ")})`,
      example: value === undefined || value === null ? undefined : exampleOf(value, call),
      element: elementwise.includes(filter),
    };
  });
}
