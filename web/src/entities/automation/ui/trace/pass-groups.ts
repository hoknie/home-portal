import type { TraceEntry } from "../../model/schema";

export type TraceRow =
  | { type: "entry"; entry: TraceEntry; index: number }
  | { type: "pass"; loop: string; number: number; item: string | null; depth: number }
  | { type: "branch"; parallel: string; number: number; depth: number };

type Indexed = { entry: TraceEntry; index: number };

const BODY = ".body[";

export function depthOf(path: string) {
  return Math.max(0, (path.match(/\[\d+\]/g)?.length ?? 1) - 1);
}

export function loopOf(path: string): string | null {
  const at = path.lastIndexOf(BODY);
  return at < 0 ? null : path.slice(0, at);
}

function branchOf(path: string, parallel: string): number {
  return Number(/^\[(\d+)\]/.exec(path.slice(`${parallel}.branches`.length))?.[1] ?? 0);
}

function arranged(items: Indexed[]): (Indexed | Extract<TraceRow, { type: "branch" }>)[] {
  const out: (Indexed | Extract<TraceRow, { type: "branch" }>)[] = [];
  let at = 0;
  while (at < items.length) {
    const item = items[at];
    out.push(item);
    at += 1;
    if (item.entry.kind !== "parallel") {
      continue;
    }
    const prefix = `${item.entry.path}.branches[`;
    const inside: Indexed[] = [];
    while (at < items.length && items[at].entry.path.startsWith(prefix)) {
      inside.push(items[at]);
      at += 1;
    }
    const branches = new Map<number, Indexed[]>();
    for (const nested of inside) {
      const number = branchOf(nested.entry.path, item.entry.path);
      branches.set(number, [...(branches.get(number) ?? []), nested]);
    }
    for (const [number, members] of [...branches.entries()].sort(([left], [right]) => left - right)) {
      out.push({ type: "branch", parallel: item.entry.path, number: number + 1, depth: depthOf(item.entry.path) + 1 });
      out.push(...arranged(members));
    }
  }
  return out;
}

export function passRows(entries: TraceEntry[]): TraceRow[] {
  const shown = new Map<string, number>();
  const rows: TraceRow[] = [];
  for (const item of arranged(entries.map((entry, index) => ({ entry, index })))) {
    if ("type" in item) {
      rows.push(item);
      continue;
    }
    const { entry, index } = item;
    if (entry.kind === "loop") {
      for (const key of [...shown.keys()].filter((key) => key === entry.path || key.startsWith(`${entry.path}.`))) {
        shown.delete(key);
      }
    }
    const loop = loopOf(entry.path);
    if (loop !== null && entry.iteration !== null && shown.get(loop) !== entry.iteration) {
      shown.set(loop, entry.iteration);
      rows.push({ type: "pass", loop, number: entry.iteration + 1, item: entry.item, depth: depthOf(loop) + 1 });
    }
    rows.push({ type: "entry", entry, index });
  }
  return rows;
}

export function hasLog(entry: TraceEntry) {
  return entry.values.length > 0 || entry.log.length > 0 || entry.values_dropped > 0 || entry.log_dropped > 0;
}
