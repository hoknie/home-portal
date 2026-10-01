import { z } from "zod";

import { generated } from "@/shared/api";

export const ARGUMENT_TYPES = ["text", "number", "flag", "choice"] as const;

const served = generated.scriptsTree;

export const scriptArgumentSchema = served.argumentResponseSchema.extend({ type: z.enum(ARGUMENT_TYPES) });

export type ScriptArgument = z.infer<typeof scriptArgumentSchema>;

export const headerProblemSchema = served.headerProblemResponseSchema;

export type HeaderProblem = z.infer<typeof headerProblemSchema>;

export const scriptHeaderSchema = z.object({
  description: z.string().nullable(),
  arguments: z.array(scriptArgumentSchema),
  problems: z.array(headerProblemSchema),
});

export type ScriptHeader = z.infer<typeof scriptHeaderSchema>;

export const UNREADABLE = ["binary", "too-large", "link"] as const;

export const scriptEntrySchema = served.scriptFileResponseSchema.extend({ unreadable: z.enum(UNREADABLE).nullable(), arguments: z.array(scriptArgumentSchema) });

export type ScriptEntry = z.infer<typeof scriptEntrySchema>;

export const scriptTreeSchema = served.scriptTreeResponseSchema.extend({ files: z.array(scriptEntrySchema) });

export type ScriptTree = z.infer<typeof scriptTreeSchema>;

export const scriptTextSchema = generated.scriptText.scriptTextResponseSchema.extend({ entry: scriptEntrySchema });

export type ScriptText = z.infer<typeof scriptTextSchema>;
