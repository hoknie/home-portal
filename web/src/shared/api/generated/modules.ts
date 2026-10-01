import { z } from "zod";

export const moduleResponseSchema = z.object({ "enabled": z.boolean(), "name": z.string(), "required_by": z.array(z.string()), "requires": z.array(z.string()) });

export type ModuleResponse = z.infer<typeof moduleResponseSchema>;

export const modulesResponseSchema = z.object({ "modules": z.array(moduleResponseSchema) });

export type ModulesResponse = z.infer<typeof modulesResponseSchema>;

export const schema = modulesResponseSchema;
