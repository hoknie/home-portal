import type { Condition, WorkflowCatalogue } from "./schema";

export const JOINS = ["all", "any"] as const;

export type Join = (typeof JOINS)[number];

export const DEEPEST_CONDITION = 3;

export function emptyRow(): Condition {
  return { left: "", op: "==", right: "" };
}

export function joinOf(condition: Condition): Join | null {
  if (condition.all !== undefined) {
    return "all";
  }
  if (condition.any !== undefined) {
    return "any";
  }
  return null;
}

export function rowsOf(condition: Condition): Condition[] {
  return condition.all ?? condition.any ?? [condition];
}

export function joined(join: Join, rows: Condition[]): Condition {
  if (rows.length === 1 && joinOf(rows[0]) === null) {
    return rows[0];
  }
  return { [join]: rows };
}

export function takesRight(operator: string | undefined, catalogue: WorkflowCatalogue | undefined) {
  return catalogue?.operators.find((entry) => entry.name === operator)?.takes_right ?? true;
}

export function depthOf(condition: Condition): number {
  const join = joinOf(condition);
  return join === null ? 0 : 1 + Math.max(0, ...rowsOf(condition).map(depthOf));
}

export const OPERATOR_NAMES: Record<string, string> = {
  "==": "equals",
  "!=": "not-equals",
  "<": "less",
  "<=": "at-most",
  ">": "greater",
  ">=": "at-least",
  contains: "contains",
  "is-empty": "is-empty",
  "is-not-empty": "is-not-empty",
};

export function operatorName(operator: string | undefined): string {
  return OPERATOR_NAMES[operator ?? "=="] ?? operator ?? "";
}

export function summary(condition: Condition | undefined, word: (operator: string) => string = (operator) => operator): string {
  if (condition === undefined) {
    return "";
  }
  const join = joinOf(condition);
  if (join !== null) {
    return rowsOf(condition)
      .map((row) => (joinOf(row) === null ? summary(row, word) : `(${summary(row, word)})`))
      .join(join === "all" ? " && " : " || ");
  }
  return [condition.left, condition.op === undefined ? undefined : word(condition.op), condition.right].filter((part) => part !== undefined && part !== "").join(" ");
}

export function incomplete(condition: Condition | undefined, catalogue: WorkflowCatalogue | undefined): boolean {
  if (condition === undefined) {
    return true;
  }
  if (joinOf(condition) !== null) {
    const rows = rowsOf(condition);
    return rows.length === 0 || rows.some((row) => incomplete(row, catalogue));
  }
  return !condition.left || !condition.op || (takesRight(condition.op, catalogue) && condition.right === undefined);
}
