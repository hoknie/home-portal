import { z } from "zod";

import { generated } from "@/shared/api";

export const MODULE_NAMES = ["proxy", "dns", "automations", "webhooks", "users", "workflows", "notifications"] as const;

export const moduleNameSchema = z.enum(MODULE_NAMES);

export type ModuleName = z.infer<typeof moduleNameSchema>;

export const moduleSchema = generated.modules.moduleResponseSchema.extend({
  name: moduleNameSchema,
  requires: z.array(moduleNameSchema),
  required_by: z.array(moduleNameSchema),
});

export type Module = z.infer<typeof moduleSchema>;

export const modulesSchema = z.object({ modules: z.array(moduleSchema) });

export type Modules = z.infer<typeof modulesSchema>;

export type SwitchLock = { kind: "required-by"; modules: ModuleName[] } | { kind: "requires"; modules: ModuleName[] };

export function enabledModules(modules: Modules | undefined): ReadonlySet<ModuleName> {
  return new Set((modules?.modules ?? []).filter((module) => module.enabled).map((module) => module.name));
}

export function switchLock(module: Module, modules: Modules): SwitchLock | null {
  if (module.enabled) {
    return module.required_by.length > 0 ? { kind: "required-by", modules: module.required_by } : null;
  }
  const on = enabledModules(modules);
  const missing = module.requires.filter((name) => !on.has(name));
  return missing.length > 0 ? { kind: "requires", modules: missing } : null;
}
