export type FilterCall = { name: string; arguments: unknown[] };

export type ParsedChain = { filters: FilterCall[]; error: null } | { filters: null; error: string };

const FILTER_NAME = /^[a-z][a-z0-9_]*$/;

export function splitOutsideQuotes(text: string, separator: string): string[] {
  const parts: string[] = [];
  let quote: string | null = null;
  let escaped = false;
  let start = 0;
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (quote === '"' && character === "\\") {
      escaped = true;
    } else if (quote !== null && character === quote) {
      quote = null;
    } else if (quote === null && (character === '"' || character === "'")) {
      quote = character;
    } else if (quote === null && character === separator) {
      parts.push(text.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(text.slice(start));
  return parts;
}

function literal(text: string): { value: unknown } | null {
  if (text.length >= 2 && text.startsWith("'") && text.endsWith("'")) {
    return { value: text.slice(1, -1) };
  }
  try {
    const value: unknown = JSON.parse(text);
    return value === null || ["string", "number", "boolean"].includes(typeof value) ? { value } : null;
  } catch {
    return null;
  }
}

function parseCall(part: string): FilterCall | null {
  const open = part.indexOf("(");
  if (open < 0) {
    return FILTER_NAME.test(part) ? { name: part, arguments: [] } : null;
  }
  if (!part.endsWith(")")) {
    return null;
  }
  const name = part.slice(0, open).trim();
  const inside = part.slice(open + 1, -1);
  const values = inside.trim() === "" ? [] : splitOutsideQuotes(inside, ",").map((argument) => literal(argument.trim()));
  if (!FILTER_NAME.test(name) || values.some((value) => value === null)) {
    return null;
  }
  return { name, arguments: values.map((value) => value!.value) };
}

export function parseChain(text: string): ParsedChain {
  const filters: FilterCall[] = [];
  for (const raw of splitOutsideQuotes(text, "|")) {
    const part = raw.trim();
    const call = parseCall(part);
    if (call === null) {
      return { filters: null, error: part };
    }
    filters.push(call);
  }
  return { filters, error: null };
}
