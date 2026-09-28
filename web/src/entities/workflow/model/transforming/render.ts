import type { Condition } from "../schema";
import { applyChain } from "./filters";
import { type FilterCall, parseChain } from "./parse";
import { textOf, typeOfValue, walk } from "./values";

export const NAMESPACES = ["event", "inputs", "vars", "steps", "loop", "secrets", "item", "index", "portal"] as const;

export const BARE_NAMESPACES = ["item", "index"];

export type Placeholder = { name: string; filters: FilterCall[] | null; filterError: string | null; length: number };

export type Lookup = (name: string) => unknown;

const PART = /^[A-Za-z0-9_-]+$/;

export function validName(name: string) {
  const parts = name.split(".");
  return (
    (NAMESPACES as readonly string[]).includes(parts[0]) &&
    (parts.length > 1 || BARE_NAMESPACES.includes(parts[0])) &&
    parts.slice(1).every((part) => PART.test(part))
  );
}

export function placeholderAt(text: string): Placeholder | null {
  const end = text.indexOf("}}");
  if (end < 0) {
    return null;
  }
  const inner = text.slice(0, end);
  const bar = inner.indexOf("|");
  const name = bar < 0 ? inner : inner.slice(0, bar).trim();
  if (!validName(name)) {
    return null;
  }
  const chain = bar < 0 ? { filters: [], error: null } : parseChain(inner.slice(bar + 1));
  return { name, filters: chain.filters, filterError: chain.error, length: end };
}

function evaluate(placeholder: Placeholder, lookup: Lookup): unknown {
  if (placeholder.filters === null) {
    throw new Error(`{{${placeholder.name}}} has a filter that cannot be read: "${placeholder.filterError ?? ""}"`);
  }
  return applyChain(lookup(placeholder.name), placeholder.filters);
}

export function renderText(template: string, lookup: Lookup): string {
  let rendered = "";
  let rest = template;
  for (;;) {
    const start = rest.indexOf("{{");
    if (start < 0) {
      return rendered + rest;
    }
    const after = rest.slice(start + 2);
    const placeholder = placeholderAt(after);
    if (placeholder === null) {
      rendered += rest.slice(0, start + 1);
      rest = rest.slice(start + 1);
    } else {
      rendered += rest.slice(0, start) + textOf(evaluate(placeholder, lookup));
      rest = after.slice(placeholder.length + 2);
    }
  }
}

export function renderValue(template: string, lookup: Lookup): unknown {
  const trimmed = template.trim();
  if (trimmed.startsWith("{{") && trimmed.endsWith("}}")) {
    const placeholder = placeholderAt(trimmed.slice(2));
    if (placeholder !== null && trimmed.length === placeholder.length + 4) {
      return evaluate(placeholder, lookup);
    }
  }
  return renderText(template, lookup);
}

function numberOf(text: string): number | null {
  const number = Number(text.trim());
  return text.trim() !== "" && Number.isFinite(number) && /^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(text.trim()) ? number : null;
}

function empty(value: unknown) {
  switch (typeOfValue(value)) {
    case "null":
      return true;
    case "text":
      return (value as string).trim() === "";
    case "list":
      return (value as unknown[]).length === 0;
    case "object":
      return Object.keys(value as object).length === 0;
    default:
      return false;
  }
}

export function compare(left: unknown, operator: string, right: string): boolean {
  const text = textOf(left);
  switch (operator) {
    case "==":
      return text === right;
    case "!=":
      return text !== right;
    case "<":
    case "<=":
    case ">":
    case ">=": {
      const [a, b] = [numberOf(text), numberOf(right)];
      if (a === null || b === null) {
        return false;
      }
      return operator === "<" ? a < b : operator === "<=" ? a <= b : operator === ">" ? a > b : a >= b;
    }
    case "contains":
      return Array.isArray(left) ? left.some((item) => textOf(item) === right) : text.includes(right);
    case "is-empty":
      return empty(left);
    case "is-not-empty":
      return !empty(left);
    default:
      return false;
  }
}

export function holds(condition: Condition, lookup: Lookup): boolean {
  if (condition.all) {
    return condition.all.every((inner) => holds(inner, lookup));
  }
  if (condition.any) {
    return condition.any.some((inner) => holds(inner, lookup));
  }
  return compare(renderValue(condition.left ?? "", lookup), condition.op ?? "==", renderText(condition.right ?? "", lookup));
}

export function itemLookup(item: unknown, index: number, fallback: Lookup): Lookup {
  return (name) => {
    const [namespace, ...path] = name.split(".");
    if (namespace === "item") {
      return walk(item, path);
    }
    return namespace === "index" ? index : fallback(name);
  };
}
