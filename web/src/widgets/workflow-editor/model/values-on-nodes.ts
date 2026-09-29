import type { TraceEntry } from "@/entities/automation";
import type { Step } from "@/entities/workflow";

export const MARK_OPEN = "";
export const MARK_SPLIT = "";
export const MARK_CLOSE = "";

const OWN_FIELDS = new Set(["id", "kind", "label"]);
const CHILD_STEPS = new Set(["then", "else", "body", "branches"]);
const MARKED = new RegExp(`${MARK_OPEN}(\\d+)${MARK_SPLIT}([^${MARK_CLOSE}]*)${MARK_CLOSE}`, "g");

export type Shown = { template: string; value: string };

export type Part = { text: string } | { text: string; template: string };

function readable(value: string) {
  try {
    const parsed: unknown = JSON.parse(value);
    return typeof parsed === "string" ? parsed : JSON.stringify(parsed);
  } catch {
    return value;
  }
}

export function withValues(step: Step, values: TraceEntry["values"]): { step: Step; shown: Shown[] } {
  const known = new Map(values.map((entry) => [entry.template, entry.value]));
  const shown: Shown[] = [];
  const swap = (value: unknown): unknown => {
    if (typeof value === "string") {
      const found = known.get(value);
      if (found === undefined) {
        return value;
      }
      shown.push({ template: value, value: readable(found) });
      return `${MARK_OPEN}${shown.length - 1}${MARK_SPLIT}${readable(found)}${MARK_CLOSE}`;
    }
    if (Array.isArray(value)) {
      return value.map(swap);
    }
    if (value !== null && typeof value === "object") {
      return Object.fromEntries(Object.entries(value).map(([key, inner]) => [key, swap(inner)]));
    }
    return value;
  };
  const copy = Object.fromEntries(Object.entries(step).map(([key, value]) => [key, OWN_FIELDS.has(key) || CHILD_STEPS.has(key) ? value : swap(value)]));
  return { step: copy as Step, shown };
}

const STRAY = new RegExp(`[${MARK_OPEN}${MARK_SPLIT}${MARK_CLOSE}]`, "g");

function plain(text: string) {
  return text.replace(STRAY, "");
}

export const SHOW_VALUES_KEY = "home-portal.workflow-editor.show-values";

export function showValuesRemembered() {
  try {
    return window.localStorage.getItem(SHOW_VALUES_KEY) !== "0";
  } catch {
    return true;
  }
}

export function rememberShowValues(show: boolean) {
  try {
    window.localStorage.setItem(SHOW_VALUES_KEY, show ? "1" : "0");
  } catch {
    return;
  }
}

export function partsOf(text: string, shown: Shown[]): Part[] {
  const parts: Part[] = [];
  let last = 0;
  for (const match of text.matchAll(MARKED)) {
    if (match.index > last) {
      parts.push({ text: plain(text.slice(last, match.index)) });
    }
    parts.push({ text: match[2], template: shown[Number(match[1])]?.template ?? "" });
    last = match.index + match[0].length;
  }
  if (last < text.length) {
    parts.push({ text: plain(text.slice(last)) });
  }
  return parts;
}

export function valuesOf(entries: TraceEntry[]) {
  return [...entries].reverse().find((entry) => entry.values.length > 0)?.values ?? [];
}
