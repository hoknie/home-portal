import { z } from "zod";

import { ADMIN_GROUP } from "./schema";

export type Rights = Record<string, string[]>;

export const groupSchema = z.object({
  name: z.string(),
  builtin: z.boolean(),
  rights: z.record(z.string(), z.array(z.string())),
  members: z.array(z.string()),
});

export type Group = z.infer<typeof groupSchema>;

export const matrixRowSchema = z.object({ area: z.string(), actions: z.array(z.string()) });

export type MatrixRow = z.infer<typeof matrixRowSchema>;

export const groupsSchema = z.object({ groups: z.array(groupSchema), matrix: z.array(matrixRowSchema) });

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
