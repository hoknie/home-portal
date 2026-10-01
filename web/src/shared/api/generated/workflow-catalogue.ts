import { z } from "zod";

export const eventFieldsResponseSchema = z.object({ "fields": z.array(z.string()), "name": z.string() });

export type EventFieldsResponse = z.infer<typeof eventFieldsResponseSchema>;

export const argumentResponseSchema = z.object({ "choices": z.array(z.string()), "name": z.string(), "required": z.boolean(), "type": z.string() });

export type ArgumentResponse = z.infer<typeof argumentResponseSchema>;

export const filterResponseSchema = z.object({ "accepts": z.array(z.string()), "arguments": z.array(argumentResponseSchema), "element": z.boolean(), "gives": z.string(), "name": z.string() });

export type FilterResponse = z.infer<typeof filterResponseSchema>;

export const stepFieldResponseSchema = z.object({ "choices": z.array(z.string()), "default": z.string().nullable(), "maximum": z.number().nullable(), "minimum": z.number().nullable(), "name": z.string(), "required": z.boolean(), "template_keys": z.boolean(), "templated": z.boolean(), "type": z.string() });

export type StepFieldResponse = z.infer<typeof stepFieldResponseSchema>;

export const stepKindResponseSchema = z.object({ "exclusive": z.array(z.array(z.string())), "fields": z.array(stepFieldResponseSchema), "group": z.string(), "name": z.string(), "results": z.array(z.string()) });

export type StepKindResponse = z.infer<typeof stepKindResponseSchema>;

export const operatorResponseSchema = z.object({ "name": z.string(), "takes_right": z.boolean() });

export type OperatorResponse = z.infer<typeof operatorResponseSchema>;

export const workflowCatalogueResponseSchema = z.object({ "events": z.array(eventFieldsResponseSchema), "filters": z.array(filterResponseSchema), "kinds": z.array(stepKindResponseSchema), "operations": z.array(filterResponseSchema), "operators": z.array(operatorResponseSchema) });

export type WorkflowCatalogueResponse = z.infer<typeof workflowCatalogueResponseSchema>;

export const schema = workflowCatalogueResponseSchema;
