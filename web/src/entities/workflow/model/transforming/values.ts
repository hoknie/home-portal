export const VALUE_TYPES = ["text", "number", "boolean", "list", "object", "null", "any"] as const;

export type ValueType = (typeof VALUE_TYPES)[number];


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
