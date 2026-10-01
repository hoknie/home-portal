import { z } from "zod";

export const outputResponseSchema = z.object({ "bytes": z.number(), "tail": z.string(), "truncated": z.boolean() });

export type OutputResponse = z.infer<typeof outputResponseSchema>;

export const outcomeResponseSchema = z.object({ "count": z.number(), "duration_milliseconds": z.number(), "exit_code": z.number().nullable(), "last_at": z.string(), "reason": z.string().nullable(), "result": z.string(), "stderr": outputResponseSchema, "stdout": outputResponseSchema });

export type OutcomeResponse = z.infer<typeof outcomeResponseSchema>;

export const renderedResponseSchema = z.object({ "template": z.string(), "value": z.string() });

export type RenderedResponse = z.infer<typeof renderedResponseSchema>;

export const traceEntryResponseSchema = z.object({ "budget_reached": z.boolean(), "command": z.array(z.string()).nullable(), "detail": z.string(), "duration_milliseconds": z.number(), "item": z.string().nullable(), "iteration": z.number().nullable(), "kind": z.string(), "label": z.string(), "level": z.string().nullable(), "log": z.array(z.string()), "log_dropped": z.number(), "outcome": z.string(), "output": z.string().nullable(), "path": z.string(), "shape": z.string().nullable(), "started_at": z.string(), "stderr": outputResponseSchema.nullable(), "stdout": outputResponseSchema.nullable(), "step": z.string(), "values": z.array(renderedResponseSchema), "values_dropped": z.number(), "wait_seconds": z.number().nullable() });

export type TraceEntryResponse = z.infer<typeof traceEntryResponseSchema>;

export const traceResponseSchema = z.object({ "dropped": z.number(), "entries": z.array(traceEntryResponseSchema) });

export type TraceResponse = z.infer<typeof traceResponseSchema>;

export const runResponseSchema = z.object({ "arguments": z.array(z.string()), "automation": z.string(), "event": z.string(), "fields": z.record(z.string(), z.string()), "id": z.string(), "outcome": outcomeResponseSchema, "started_at": z.string(), "steps_version": z.string().nullable(), "trace": traceResponseSchema.nullable(), "workflow": z.string().nullable() });

export type RunResponse = z.infer<typeof runResponseSchema>;

export const runsResponseSchema = z.object({ "runs": z.array(runResponseSchema) });

export type RunsResponse = z.infer<typeof runsResponseSchema>;

export const schema = runsResponseSchema;
