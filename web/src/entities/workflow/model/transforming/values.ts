export const VALUE_TYPES = ["text", "number", "boolean", "list", "object", "null", "any"] as const;

export type ValueType = (typeof VALUE_TYPES)[number];

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

const DESCRIBED: Record<ValueType, string> = {
  text: "text",
  number: "a number",
  boolean: "a boolean",
  list: "a list",
  object: "an object",
  null: "nothing",
  any: "any value",
};

export function described(type: ValueType): string {
  return DESCRIBED[type];
}

export function typeOfValue(value: unknown): ValueType {
  if (value === null || value === undefined) {
    return "null";
  }
  if (Array.isArray(value)) {
    return "list";
  }
  switch (typeof value) {
    case "string":
      return "text";
    case "number":
      return "number";
    case "boolean":
      return "boolean";
    default:
      return "object";
  }
}

export function isObject(value: unknown): value is Record<string, unknown> {
  return typeOfValue(value) === "object";
}

export function canonical(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map(canonical).join(",")}]`;
  }
  if (isObject(value)) {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value ?? null);
}

export function textOf(value: unknown): string {
  if (value === null || value === undefined) {
    return "";
  }
  return typeof value === "string" ? value : canonical(value);
}

export function numberValue(number: number): number | null {
  return Number.isFinite(number) ? number : null;
}

export function walk(value: unknown, path: string[]): unknown {
  return path.reduce<unknown>((current, part) => {
    if (Array.isArray(current)) {
      return /^\d+$/.test(part) && Number(part) < current.length ? current[Number(part)] : null;
    }
    if (isObject(current)) {
      if (part in current) {
        return current[part];
      }
      const lowered = part.toLowerCase();
      const key = Object.keys(current).find((candidate) => candidate.toLowerCase() === lowered);
      return key === undefined ? null : current[key];
    }
    return null;
  }, value);
}

export function atKey(value: unknown, key: string): unknown {
  return walk(value, key.split(".").filter((part) => part !== ""));
}

function rank(value: unknown): number {
  return ["null", "boolean", "number", "text", "list", "object"].indexOf(typeOfValue(value));
}

export function orderOf(left: unknown, right: unknown): number {
  if (typeof left === "number" && typeof right === "number") {
    return left < right ? -1 : left > right ? 1 : 0;
  }
  const byRank = rank(left) - rank(right);
  if (byRank !== 0) {
    return byRank;
  }
  const leftText = textOf(left);
  const rightText = textOf(right);
  return leftText < rightText ? -1 : leftText > rightText ? 1 : 0;
}

export function sameValue(left: unknown, right: unknown) {
  return canonical(left) === canonical(right);
}
