import type { Trace } from "@/entities/automation/@x/workflow";

import type { FilterDescription, InputDeclaration, Step, WorkflowCatalogue } from "../schema";
import { type PortalValues, portalValue } from "../portal";
import { scopeAt } from "../scope";
import { lastOutput } from "../suggestions/last-output";
import { type Path, at, everyStep } from "../tree";
import type { Operation } from "./operations";
import { parseChain } from "./parse";
import { type ChainPreview, type PreviewCache, askPreview, settled } from "./previews";
import { placeholderAt } from "./render";
import { certainType, givesAfter } from "./types";
import { type ValueType, textOf, typeOfValue, walk } from "./values";

export type KnownContext = { steps: Step[]; inputs: (string | InputDeclaration)[]; path: Path; field: string; lastRun: Trace | null; catalogue: WorkflowCatalogue | undefined; portal?: PortalValues | null; preview?: PreviewCache };

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

function inputParts(step: Step): { name: string; filters: string } | null {
  const trimmed = (step.input ?? "").trim();
  const placeholder = trimmed.startsWith("{{") ? placeholderAt(trimmed.slice(2)) : null;
  if (placeholder === null || placeholder.filters === null || trimmed.length !== placeholder.length + 4) {
    return null;
  }
  const inner = trimmed.slice(2, -2);
  const bar = inner.indexOf("|");
  return { name: placeholder.name, filters: bar < 0 ? "" : inner.slice(bar) };
}

export function previewChain(value: unknown, operations: Operation[], context: KnownContext, depth = 0, filters = "", examples: string[] = []): ChainPreview {
  return askPreview(context.preview, { value, filters, operations, examples }, (name) => sampleOf(name, context, depth + 1));
}

export function inputSample(step: Step, context: KnownContext, depth = 0): Sample | null {
  const parts = inputParts(step);
  const source = parts === null ? null : sampleOf(parts.name, context, depth + 1);
  if (parts === null || source === null) {
    return null;
  }
  if (parts.filters === "") {
    return source;
  }
  const input = previewChain(source.value, [], context, depth, parts.filters).input;
  return settled(input) ? { value: input.value, from: source.from } : null;
}

function afterChain(step: Step, context: KnownContext, depth: number, count: number): Sample | null {
  const input = inputSample(step, context, depth);
  if (input === null || count === 0) {
    return input;
  }
  const last = previewChain(input.value, step.operations ?? [], context, depth).steps[count - 1];
  return settled(last) ? { value: last.value, from: input.from } : null;
}

export function sampleOf(name: string, context: KnownContext, depth = 0): Sample | null {
  const [namespace, id = "", field = "", ...path] = name.split(".");
  if (namespace === "portal") {
    const value = portalValue(name, context.portal);
    return value === undefined ? null : { value, from: "sample" };
  }
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
    const after = afterChain(step, context, depth, (step.operations ?? []).length);
    if (after !== null) {
      return { value: walk(after.value, path), from: after.from };
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
  return step?.kind === "transform" ? afterChain(step, context, 0, index) : null;
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

function exampleOf(preview: ChainPreview["examples"][number] | undefined): string | undefined {
  if (!settled(preview) || preview.value === null) {
    return undefined;
  }
  const text = textOf(preview.value);
  return text.length > LONGEST_EXAMPLE ? `${text.slice(0, LONGEST_EXAMPLE - 1)}…` : text;
}

export function filterOffers(context: KnownContext, subject: string, chain: string): FilterOffer[] {
  const before = chain === "" ? [] : (parseChain(chain).filters ?? []);
  const type = givesAfter(knownType(subject, context), before, context.catalogue?.filters ?? []);
  const sample = subject.startsWith("item") ? itemSample(context) : sampleOf(subject, context);
  const filters = context.catalogue?.filters ?? [];
  const exact = filters.filter((filter) => type === "any" || filter.accepts.includes(type));
  const general = type === "any" ? [] : filters.filter((filter) => !exact.includes(filter) && filter.accepts.includes("any"));
  const elements = type === "list" ? filters.filter((filter) => filter.element && !exact.includes(filter)) : [];
  const candidates = [...exact, ...elements, ...general];
  const insertOf = (filter: FilterDescription) => {
    const required = filter.arguments.filter((argument) => argument.required);
    return required.length === 0 ? filter.name : `${filter.name}(${required.map(placeholderOf).join(", ")})`;
  };
  const start = sample === null ? undefined : subject.startsWith("item") ? walk(sample.value, subject.split(".").slice(1)) : sample.value;
  const asked = start === undefined || start === null ? null : previewChain(start, [], context, 0, chain === "" ? "" : `| ${chain}`, candidates.map(insertOf));
  const value = chain === "" ? start : asked !== null && settled(asked.input) ? asked.input.value : undefined;
  const first = Array.isArray(value) && value.length > 0 ? typeOfValue(value[0]) : null;
  const elementwise = elements.filter((filter) => first === null || filter.accepts.includes(first as never));
  return candidates
    .filter((filter) => !elements.includes(filter) || elementwise.includes(filter))
    .map((filter) => ({
      name: filter.name,
      insert: insertOf(filter),
      label: filter.arguments.length === 0 ? filter.name : `${filter.name}(${filter.arguments.map((argument) => argument.name).join(", ")})`,
      example: asked === null ? undefined : exampleOf(asked.examples[candidates.indexOf(filter)]),
      element: elements.includes(filter),
    }));
}
