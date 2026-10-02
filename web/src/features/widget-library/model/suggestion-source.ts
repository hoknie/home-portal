import type { TemplateSuggestion } from "@/shared/ui/template-input";

import { DEEPEST_PATH, MOST_PATHS, firstItemOf } from "./data-paths";

export type DeclaredPath = { path: string; description: string | null; kind: string };

export type KnownPath = { path: string; kind: string; description: string | null; sample: string | null };

export type Groups = { data: string; widget: string; item: string };

export const WIDGET_NAMES = ["widget.id", "widget.title", "fetched_at"] as const;

export const LONGEST_SAMPLE = 40;

export function kindOf(value: unknown): string {
  if (value === null || value === undefined) {
    return "value";
  }
  if (Array.isArray(value)) {
    return "list";
  }
  return typeof value === "object" ? "object" : typeof value === "string" ? "text" : typeof value;
}

function sampleOf(value: unknown): string | null {
  if (value === null || value === undefined || typeof value === "object") {
    return null;
  }
  const text = String(value);
  return text.length > LONGEST_SAMPLE ? `${text.slice(0, LONGEST_SAMPLE - 1)}…` : text;
}

function walked(value: unknown, prefix: string, depth = 0, found: KnownPath[] = []): KnownPath[] {
  if (found.length >= MOST_PATHS) {
    return found;
  }
  found.push({ path: prefix, kind: kindOf(value), description: null, sample: sampleOf(value) });
  if (depth >= DEEPEST_PATH || value === null || typeof value !== "object") {
    return found;
  }
  const children: [string, unknown][] = Array.isArray(value) ? (value.length > 0 ? [["0", value[0]]] : []) : Object.entries(value);
  for (const [key, inner] of children) {
    walked(inner, `${prefix}.${key}`, depth + 1, found);
  }
  return found;
}

export function knownPaths(declared: DeclaredPath[], data: unknown): KnownPath[] {
  const seen = data === null || data === undefined ? [] : walked(data, "data").filter((known) => known.path !== "data");
  const byPath = new Map(seen.map((known) => [known.path, known]));
  const fromDeclared = declared.map<KnownPath>((path) => {
    const live = byPath.get(path.path);
    return { path: path.path, description: path.description, kind: live?.kind ?? path.kind, sample: live?.sample ?? null };
  });
  const named = new Set(fromDeclared.map((known) => known.path));
  return [...fromDeclared, ...seen.filter((known) => !named.has(known.path))];
}

export function itemPaths(items: string | null, data: unknown, known: KnownPath[]): KnownPath[] {
  if (items === null) {
    return [];
  }
  const index: KnownPath = { path: "index", kind: "number", description: null, sample: null };
  const first = firstItemOf(data, items);
  if (first !== undefined) {
    return [...walked(first, "item"), index];
  }
  const list = /^\s*\{\{\s*(data(?:\.[A-Za-z0-9_-]+)*)\s*\}\}\s*$/.exec(items)?.[1];
  if (list === undefined) {
    return [{ path: "item", kind: "value", description: null, sample: null }, index];
  }
  const inside = known
    .filter((path) => path.path.startsWith(`${list}.0`))
    .map((path) => ({ ...path, path: `item${path.path.slice(`${list}.0`.length)}` }));
  return [...(inside.length > 0 ? inside : [{ path: "item", kind: "value", description: null, sample: null }]), index];
}

export function suggestionsOf(known: KnownPath[], items: KnownPath[], groups: Groups, kindLabel: (kind: string) => string): TemplateSuggestion[] {
  const describe = (path: KnownPath) => [path.description, kindLabel(path.kind)].filter(Boolean).join(" · ");
  return [
    ...items.map((path) => ({ value: path.path, group: groups.item, description: describe(path), example: path.sample ?? undefined })),
    ...known.map((path) => ({ value: path.path, group: groups.data, description: describe(path), example: path.sample ?? undefined })),
    ...WIDGET_NAMES.map((value) => ({ value, group: groups.widget })),
  ];
}
