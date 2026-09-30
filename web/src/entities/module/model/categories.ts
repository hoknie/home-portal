import type { ModuleName } from "./schema";

export const MODULE_CATEGORIES = ["network", "automation", "notifications", "access"] as const;

export type ModuleCategory = (typeof MODULE_CATEGORIES)[number];

export const CATEGORY_PARAMETER = "category";

export const CATEGORY_OF: Record<ModuleName, ModuleCategory> = {
  proxy: "network",
  dns: "network",
  automations: "automation",
  webhooks: "automation",
  workflows: "automation",
  notifications: "notifications",
  users: "access",
};

export function isCategory(value: unknown): value is ModuleCategory {
  return typeof value === "string" && (MODULE_CATEGORIES as readonly string[]).includes(value);
}

export function categoryParameter(value: string | null | undefined): ModuleCategory | null {
  return isCategory(value) ? value : null;
}
