import type { Run, TraceEntry } from "./schema";

type Output = Run["outcome"]["stdout"];

export type ScriptLog =
  | { kind: "streams"; stdout: Output; stderr: Output; command: string[] | null; budgetReached: boolean }
  | { kind: "merged"; output: Output };

const EXIT_CODE = /\bexited (-?\d+)(?::|$)/;

export function hasScriptLog(entry: TraceEntry) {
  return entry.kind === "script" && (entry.stdout !== null || entry.stderr !== null || entry.output !== null);
}

export function scriptLogOf(entry: TraceEntry): ScriptLog | null {
  if (entry.stdout !== null || entry.stderr !== null) {
    const empty = { tail: "", bytes: 0, truncated: false };
    return { kind: "streams", stdout: entry.stdout ?? empty, stderr: entry.stderr ?? empty, command: entry.command, budgetReached: entry.budget_reached };
  }
  if (entry.output !== null) {
    return { kind: "merged", output: { tail: entry.output, bytes: new TextEncoder().encode(entry.output).length, truncated: false } };
  }
  return null;
}

export function exitCodeOf(entry: TraceEntry) {
  const found = EXIT_CODE.exec(entry.detail);
  return found ? Number(found[1]) : null;
}
