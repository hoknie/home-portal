import { z } from "zod";

export const previewStepResponseSchema = z.object({ "error": z.string().nullable().default(null), "value": z.unknown().optional() });

export type PreviewStepResponse = z.infer<typeof previewStepResponseSchema>;

export const transformPreviewResponseSchema = z.object({ "examples": z.array(previewStepResponseSchema), "input": previewStepResponseSchema, "steps": z.array(previewStepResponseSchema) });

export type TransformPreviewResponse = z.infer<typeof transformPreviewResponseSchema>;

export const schema = transformPreviewResponseSchema;
