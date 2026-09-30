import { z } from "zod";

import { type InputDeclaration, fitsType } from "@/entities/workflow";

import { unknownPlaceholders } from "./placeholders";

export const inputEntrySchema = z.object({ template: z.boolean(), text: z.string(), value: z.unknown() });

export type InputEntry = z.infer<typeof inputEntrySchema>;

export type WorkflowCallFields = { workflow: string; inputs: Record<string, InputEntry> };

export const EMPTY_ENTRY: InputEntry = { template: true, text: "", value: null };

export function emptyEntry(declaration: InputDeclaration | undefined): InputEntry {
  return (declaration?.type ?? "text") === "text" ? EMPTY_ENTRY : { template: false, text: "", value: null };
}

export function entryOf(stored: unknown): InputEntry {
  return typeof stored === "string" ? { template: true, text: stored, value: null } : { template: false, text: "", value: stored };
}

export function entriesOf(stored: Record<string, unknown> | undefined): Record<string, InputEntry> {
  return Object.fromEntries(Object.entries(stored ?? {}).map(([name, value]) => [name, entryOf(value)]));
}

export function templated(entry: InputEntry, declaration: InputDeclaration | undefined) {
  return entry.template || (declaration?.type ?? "text") === "text";
}

function blank(value: unknown) {
  return value === null || value === undefined || value === "" || (Array.isArray(value) && value.length === 0) || (typeof value === "object" && value !== null && !Array.isArray(value) && Object.keys(value).length === 0);
}

export function textOf(entry: InputEntry) {
  if (entry.template || blank(entry.value)) {
    return entry.text;
  }
  return typeof entry.value === "string" ? entry.value : JSON.stringify(entry.value);
}

export function switched(entry: InputEntry, declaration: InputDeclaration | undefined): InputEntry {
  if (!entry.template) {
    return { template: true, text: textOf(entry), value: entry.value };
  }
  const type = declaration?.type ?? "text";
  let parsed: unknown = null;
  try {
    parsed = JSON.parse(entry.text) as unknown;
  } catch {
    parsed = null;
  }
  return { template: false, text: entry.text, value: fitsType(type, parsed) ? parsed : null };
}

export function inputsRequest(inputs: Record<string, InputEntry>, declarations: readonly InputDeclaration[] | undefined): Record<string, unknown> {
  return Object.fromEntries(
    Object.entries(inputs).flatMap(([name, entry]) => {
      const declaration = declarations?.find((input) => input.name === name);
      if (declarations !== undefined && declaration === undefined) {
        return [];
      }
      if (templated(entry, declaration)) {
        return entry.text === "" ? [] : [[name, entry.text]];
      }
      return blank(entry.value) ? [] : [[name, entry.value]];
    }),
  );
}

export function kept(inputs: Record<string, InputEntry>, declarations: readonly InputDeclaration[]) {
  return Object.fromEntries(Object.entries(inputs).filter(([name]) => declarations.some((input) => input.name === name)));
}

export function inputPlaceholderIssues(inputs: Record<string, InputEntry>, allowed: readonly string[]) {
  return Object.entries(inputs)
    .filter(([, entry]) => entry.template && unknownPlaceholders(entry.text, allowed).length > 0)
    .map(([name]) => name);
}
