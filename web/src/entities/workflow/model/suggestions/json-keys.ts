export const LONGEST_EXAMPLE = 40;
export const DEEPEST_KEY = 2;

export type JsonKey = { path: string; example: string };

function exampleOf(value: unknown): string {
  const text = typeof value === "string" ? value : Array.isArray(value) ? `[${value.length}]` : value !== null && typeof value === "object" ? "{…}" : JSON.stringify(value);
  return text.length > LONGEST_EXAMPLE ? `${text.slice(0, LONGEST_EXAMPLE - 1)}…` : text;
}

function walk(value: unknown, prefix: string, depth: number, found: JsonKey[]) {
  if (value === null || typeof value !== "object" || Array.isArray(value) || depth > DEEPEST_KEY) {
    return;
  }
  for (const [key, inner] of Object.entries(value as Record<string, unknown>)) {
    if (!/^[A-Za-z0-9_-]+$/.test(key)) {
      continue;
    }
    const path = prefix === "" ? key : `${prefix}.${key}`;
    found.push({ path, example: exampleOf(inner) });
    walk(inner, path, depth + 1, found);
  }
}

export function jsonKeys(output: string | null | undefined): JsonKey[] {
  if (!output) {
    return [];
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(output);
  } catch {
    return [];
  }
  const found: JsonKey[] = [];
  walk(parsed, "", 1, found);
  return found;
}
