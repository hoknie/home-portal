export const DEEPEST_PATH = 4;
export const MOST_PATHS = 80;

export function pathsOf(value: unknown, prefix: string, depth = 0, found: string[] = []): string[] {
  if (found.length >= MOST_PATHS) {
    return found;
  }
  found.push(prefix);
  if (depth >= DEEPEST_PATH || value === null || typeof value !== "object") {
    return found;
  }
  if (Array.isArray(value)) {
    if (value.length > 0) {
      pathsOf(value[0], `${prefix}.0`, depth + 1, found);
    }
    return found;
  }
  for (const [key, inner] of Object.entries(value)) {
    pathsOf(inner, `${prefix}.${key}`, depth + 1, found);
  }
  return found;
}

export function firstItemOf(value: unknown, listTemplate: string): unknown {
  const match = /^\s*\{\{\s*data((?:\.[A-Za-z0-9_-]+)*)\s*\}\}\s*$/.exec(listTemplate);
  if (!match) {
    return undefined;
  }
  let current = value;
  for (const key of match[1].split(".").filter(Boolean)) {
    current = current !== null && typeof current === "object" ? (current as Record<string, unknown>)[key] : undefined;
  }
  return Array.isArray(current) ? current[0] : undefined;
}
