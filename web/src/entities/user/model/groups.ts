import type { z } from "zod";

import { generated } from "@/shared/api";

import { ADMIN_GROUP } from "./schema";

export type Rights = Record<string, string[]>;

export const groupSchema = generated.groups.groupResponseSchema;

export type Group = z.infer<typeof groupSchema>;

export const matrixRowSchema = generated.groups.areaResponseSchema;

export type MatrixRow = z.infer<typeof matrixRowSchema>;

export const groupsSchema = generated.groups.groupsResponseSchema;

export type Groups = z.infer<typeof groupsSchema>;

export function within(rights: Rights, held: Rights) {
  return Object.entries(rights).every(([area, actions]) => actions.every((action) => held[area]?.includes(action) ?? false));
}

export function givable(groups: Groups, held: Rights, admin: boolean): string[] {
  if (admin) {
    return groups.groups.map((group) => group.name);
  }
  return groups.groups.filter((group) => group.name !== ADMIN_GROUP && within(group.rights, held)).map((group) => group.name);
}

export function withAction(rights: Rights, area: string, action: string, on: boolean, order: MatrixRow[]): Rights {
  const row = order.find((entry) => entry.area === area);
  const current = new Set(rights[area] ?? []);
  if (on) {
    current.add(action);
  } else {
    current.delete(action);
  }
  const actions = (row?.actions ?? []).filter((name) => current.has(name));
  const next = { ...rights };
  if (actions.length === 0) {
    delete next[area];
  } else {
    next[area] = actions;
  }
  return next;
}

export type Coverage = "all" | "some" | "none";

function coverageOf(ticked: number, total: number): Coverage {
  return ticked === 0 ? "none" : ticked === total ? "all" : "some";
}

export function withColumn(rights: Rights, action: string, on: boolean, matrix: MatrixRow[]): Rights {
  return matrix.filter((row) => row.actions.includes(action)).reduce((next, row) => withAction(next, row.area, action, on, matrix), rights);
}

export function withRow(rights: Rights, area: string, on: boolean, matrix: MatrixRow[]): Rights {
  const row = matrix.find((entry) => entry.area === area);
  return (row?.actions ?? []).reduce((next, action) => withAction(next, area, action, on, matrix), rights);
}

export function withEverything(on: boolean, matrix: MatrixRow[]): Rights {
  return on ? Object.fromEntries(matrix.filter((row) => row.actions.length > 0).map((row) => [row.area, [...row.actions]])) : {};
}

export function columnState(rights: Rights, action: string, matrix: MatrixRow[]): Coverage {
  const rows = matrix.filter((row) => row.actions.includes(action));
  return coverageOf(rows.filter((row) => rights[row.area]?.includes(action)).length, rows.length);
}

export function rowState(rights: Rights, area: string, matrix: MatrixRow[]): Coverage {
  const actions = matrix.find((entry) => entry.area === area)?.actions ?? [];
  return coverageOf(actions.filter((action) => rights[area]?.includes(action)).length, actions.length);
}

export function matrixState(rights: Rights, matrix: MatrixRow[]): Coverage {
  const total = matrix.reduce((sum, row) => sum + row.actions.length, 0);
  const ticked = matrix.reduce((sum, row) => sum + row.actions.filter((action) => rights[row.area]?.includes(action)).length, 0);
  return coverageOf(ticked, total);
}
