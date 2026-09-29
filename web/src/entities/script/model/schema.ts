import { z } from "zod";

export const ARGUMENT_TYPES = ["text", "number", "flag", "choice"] as const;

export const scriptArgumentSchema = z.object({
  name: z.string(),
  option: z.boolean(),
  required: z.boolean(),
  type: z.enum(ARGUMENT_TYPES),
  choices: z.array(z.string()).default([]),
  default: z.string().nullable().default(null),
  description: z.string().default(""),
});

export type ScriptArgument = z.infer<typeof scriptArgumentSchema>;

export const headerProblemSchema = z.object({
  line: z.number(),
  message: z.string(),
});

export type HeaderProblem = z.infer<typeof headerProblemSchema>;

export const scriptHeaderSchema = z.object({
  description: z.string().nullable(),
  arguments: z.array(scriptArgumentSchema),
  problems: z.array(headerProblemSchema),
});

export type ScriptHeader = z.infer<typeof scriptHeaderSchema>;

export const UNREADABLE = ["binary", "too-large", "link"] as const;

export const scriptEntrySchema = z.object({
  path: z.string(),
  folder: z.string().nullable(),
  name: z.string(),
  size: z.number(),
  modified: z.string().nullable(),
  mode: z.string(),
  runnable: z.boolean(),
  problem: z.string().nullable(),
  code: z.string().nullable(),
  concerns: z.string().nullable(),
  text: z.boolean(),
  unreadable: z.enum(UNREADABLE).nullable(),
  revision: z.string().nullable(),
  description: z.string().nullable(),
  arguments: z.array(scriptArgumentSchema),
  argument_problems: z.array(headerProblemSchema),
});

export type ScriptEntry = z.infer<typeof scriptEntrySchema>;

export const scriptTreeSchema = z.object({
  directory: z.string(),
  exists: z.boolean(),
  user_id: z.number(),
  left_out: z.number(),
  inside: z.boolean(),
  folders: z.array(z.string()),
  files: z.array(scriptEntrySchema),
});

export type ScriptTree = z.infer<typeof scriptTreeSchema>;

export const scriptTextSchema = z.object({
  path: z.string(),
  content: z.string(),
  revision: z.string(),
  entry: scriptEntrySchema,
});

export type ScriptText = z.infer<typeof scriptTextSchema>;
