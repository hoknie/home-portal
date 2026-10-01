import { type FilterCall, parseChain, validName } from "./parse";

export { BARE_NAMESPACES, NAMESPACES, validName } from "./parse";

export type Placeholder = { name: string; filters: FilterCall[] | null; filterError: string | null; length: number };

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
