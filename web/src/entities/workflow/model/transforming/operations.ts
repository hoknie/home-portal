import type { Condition } from "../schema";
import { applyFilter } from "./filters";
import { type Lookup, holds, itemLookup, renderValue } from "./render";
import { atKey, described, orderOf, textOf, typeOfValue } from "./values";

export type Operation = {
  op: string;
  args?: unknown[];
  where?: Condition;
  to?: string;
  key?: string;
  order?: string;
  operations?: Operation[];
};

export const LIST_OPERATIONS = ["filter", "map", "sort_by", "group_by", "count_by", "each"] as const;

export const DEEPEST_EACH = 3;

export const MOST_OPERATIONS = 20;

function listOperation(items: unknown[], operation: Operation, lookup: Lookup): unknown {
  const key = operation.key ?? "";
  switch (operation.op) {
    case "filter":
      return items.filter((item, index) => holds(operation.where ?? {}, itemLookup(item, index, lookup)));
    case "map":
      return items.map((item, index) => renderValue(operation.to ?? "", itemLookup(item, index, lookup)));
    case "sort_by": {
      const direction = operation.order === "desc" ? -1 : 1;
      return [...items].sort((left, right) => direction * orderOf(atKey(left, key), atKey(right, key)));
    }
    case "group_by": {
      const groups: Record<string, unknown[]> = {};
      for (const item of items) {
        (groups[textOf(atKey(item, key))] ??= []).push(item);
      }
      return groups;
    }
    default: {
      const counts: Record<string, number> = {};
      for (const item of items) {
        const name = textOf(atKey(item, key));
        counts[name] = (counts[name] ?? 0) + 1;
      }
      return counts;
    }
  }
}

export class OperationFailure extends Error {
  constructor(
    readonly position: string,
    readonly operation: string,
    message: string,
  ) {
    super(message);
  }
}

function each(items: unknown[], chain: Operation[], lookup: Lookup): unknown[] {
  return items.map((item, index) => {
    const inner = itemLookup(item, index, lookup);
    return chain.reduce<unknown>((value, operation, position) => {
      try {
        return applyOperation(value, operation, inner);
      } catch (error) {
        if (error instanceof OperationFailure) {
          throw new OperationFailure(`.${position + 1}${error.position}`, error.operation, error.message);
        }
        throw new OperationFailure(`.${position + 1}`, operation.op, error instanceof Error ? error.message : String(error));
      }
    }, item);
  });
}

export function applyOperation(value: unknown, operation: Operation, lookup: Lookup): unknown {
  if (!(LIST_OPERATIONS as readonly string[]).includes(operation.op)) {
    return applyFilter(value, { name: operation.op, arguments: operation.args ?? [] });
  }
  if (value === null || value === undefined) {
    return null;
  }
  if (!Array.isArray(value)) {
    throw new Error(`${operation.op} takes a list and got ${described(typeOfValue(value))}`);
  }
  return operation.op === "each" ? each(value, operation.operations ?? [], lookup) : listOperation(value, operation, lookup);
}

export type Preview = { value: unknown; error: null } | { value: null; error: string };

export function previewOperations(input: unknown, operations: Operation[], lookup: Lookup = () => null): Preview[] {
  const previews: Preview[] = [];
  let current = input;
  for (const [index, operation] of operations.entries()) {
    if (previews.at(-1)?.error) {
      previews.push({ value: null, error: previews.at(-1)!.error! });
      continue;
    }
    try {
      current = applyOperation(current, operation, lookup);
      previews.push({ value: current, error: null });
    } catch (error) {
      const position = error instanceof OperationFailure ? error.position : "";
      const name = error instanceof OperationFailure ? error.operation : operation.op;
      previews.push({ value: null, error: `operation ${index + 1}${position} (${name}): ${error instanceof Error ? error.message : String(error)}` });
    }
  }
  return previews;
}

export function transformSample(input: unknown, operations: Operation[]): { value: unknown } | { error: string } {
  const previews = previewOperations(input, operations);
  const failed = previews.find((preview) => preview.error !== null);
  if (failed) {
    return { error: failed.error! };
  }
  return { value: previews.length === 0 ? input : previews[previews.length - 1].value };
}
