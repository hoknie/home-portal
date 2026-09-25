export const SCRIPT_PROBLEMS = [
  "shape",
  "too-deep",
  "hidden",
  "not-found",
  "outside",
  "not-a-file",
  "not-executable",
  "writable",
  "owner",
  "folder-writable",
  "folder-owner",
] as const;

export type ScriptProblem = (typeof SCRIPT_PROBLEMS)[number];

export function problemOf(code: string | null): ScriptProblem | null {
  return (SCRIPT_PROBLEMS as readonly string[]).includes(code ?? "") ? (code as ScriptProblem) : null;
}

export function quotedPath(path: string) {
  return `'${path.replace(/'/g, `'\\''`)}'`;
}

export function fixOf(problem: ScriptProblem, concerns: string, userId: number): string | null {
  const path = quotedPath(concerns);
  switch (problem) {
    case "writable":
    case "folder-writable":
      return `chmod go-w ${path}`;
    case "not-executable":
      return `chmod +x ${path}`;
    case "owner":
    case "folder-owner":
      return `sudo chown ${userId} ${path}`;
    default:
      return null;
  }
}
