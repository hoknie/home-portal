import { z } from "zod";

export const headerProblemResponseSchema = z.object({ "line": z.number(), "message": z.string() });

export type HeaderProblemResponse = z.infer<typeof headerProblemResponseSchema>;

export const argumentResponseSchema = z.object({ "choices": z.array(z.string()), "default": z.string().nullable(), "description": z.string(), "name": z.string(), "option": z.boolean(), "required": z.boolean(), "type": z.string() });

export type ArgumentResponse = z.infer<typeof argumentResponseSchema>;

export const scriptFileResponseSchema = z.object({ "argument_problems": z.array(headerProblemResponseSchema), "arguments": z.array(argumentResponseSchema), "code": z.string().nullable(), "concerns": z.string().nullable(), "description": z.string().nullable(), "folder": z.string().nullable(), "mode": z.string(), "modified": z.string().nullable(), "name": z.string(), "path": z.string(), "problem": z.string().nullable(), "revision": z.string().nullable(), "runnable": z.boolean(), "size": z.number(), "text": z.boolean(), "unreadable": z.string().nullable() });

export type ScriptFileResponse = z.infer<typeof scriptFileResponseSchema>;

export const scriptTextResponseSchema = z.object({ "content": z.string(), "entry": scriptFileResponseSchema, "path": z.string(), "revision": z.string() });

export type ScriptTextResponse = z.infer<typeof scriptTextResponseSchema>;

export const schema = scriptTextResponseSchema;
