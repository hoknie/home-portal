import { z } from "zod";

export const headerProblemResponseSchema = z.object({ "line": z.number(), "message": z.string() });

export type HeaderProblemResponse = z.infer<typeof headerProblemResponseSchema>;

export const argumentResponseSchema = z.object({ "choices": z.array(z.string()), "default": z.string().nullable(), "description": z.string(), "name": z.string(), "option": z.boolean(), "required": z.boolean(), "type": z.string() });

export type ArgumentResponse = z.infer<typeof argumentResponseSchema>;

export const scriptResponseSchema = z.object({ "argument_problems": z.array(headerProblemResponseSchema), "arguments": z.array(argumentResponseSchema), "code": z.string().nullable(), "concerns": z.string().nullable(), "description": z.string().nullable(), "path": z.string(), "problem": z.string().nullable(), "runnable": z.boolean() });

export type ScriptResponse = z.infer<typeof scriptResponseSchema>;

export const scriptsResponseSchema = z.object({ "directory": z.string(), "editing": z.boolean(), "exists": z.boolean(), "scripts": z.array(scriptResponseSchema), "user_id": z.number() });

export type ScriptsResponse = z.infer<typeof scriptsResponseSchema>;

export const schema = scriptsResponseSchema;
