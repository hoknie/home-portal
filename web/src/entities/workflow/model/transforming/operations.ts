import type { Condition } from "../schema";

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
