export type Problem = { path: string; message: string };

type ErrorTree = { message?: unknown; [key: string]: unknown };

export function flatProblems(tree: unknown, prefix = ""): Problem[] {
  if (tree === null || typeof tree !== "object") {
    return [];
  }
  const node = tree as ErrorTree;
  const own = typeof node.message === "string" && prefix !== "" ? [{ path: prefix, message: node.message }] : [];
  const nested = Object.entries(node).flatMap(([key, value]) =>
    key === "message" || key === "ref" || key === "type" || key === "types" ? [] : flatProblems(value, prefix === "" ? key : `${prefix}.${key}`),
  );
  return [...own, ...nested];
}

export function hiddenProblems(problems: Problem[], shown: (path: string) => boolean) {
  return problems.filter((problem) => !shown(problem.path));
}
