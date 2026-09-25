export const PLACEHOLDER = /\{\{([a-z_][a-z0-9_]*(?:\.[a-z_][a-z0-9_]*)*)\}\}/g;

export function placeholdersOf(template: string) {
  return [...template.matchAll(PLACEHOLDER)].map((match) => match[1]);
}

export function renderTemplate(template: string, values: Record<string, string>) {
  return template.replace(PLACEHOLDER, (whole, name: string) => (Object.hasOwn(values, name) ? values[name] : whole));
}

export function unknownPlaceholders(template: string, allowed: readonly string[]) {
  return placeholdersOf(template).filter((name) => !allowed.includes(name));
}

export function tokenOf(field: string) {
  return `{{${field}}}`;
}

export function insertAt(text: string, cursor: number | null, token: string) {
  const at = cursor === null ? text.length : Math.max(0, Math.min(cursor, text.length));
  return { text: `${text.slice(0, at)}${token}${text.slice(at)}`, cursor: at + token.length };
}
