import { z } from "zod";

export const outputResponseSchema = z.object({ "bytes": z.number(), "tail": z.string(), "truncated": z.boolean() });

export type OutputResponse = z.infer<typeof outputResponseSchema>;

export const outcomeResponseSchema = z.object({ "count": z.number(), "duration_milliseconds": z.number(), "exit_code": z.number().nullable(), "last_at": z.string(), "reason": z.string().nullable(), "result": z.string(), "stderr": outputResponseSchema, "stdout": outputResponseSchema });

export type OutcomeResponse = z.infer<typeof outcomeResponseSchema>;

export const renderedResponseSchema = z.object({ "template": z.string(), "value": z.string() });

export type RenderedResponse = z.infer<typeof renderedResponseSchema>;

export const traceEntryResponseSchema = z.object({ "budget_reached": z.boolean(), "command": z.array(z.string()).nullable(), "detail": z.string(), "duration_milliseconds": z.number(), "item": z.string().nullable(), "iteration": z.number().nullable(), "kind": z.string(), "label": z.string(), "level": z.string().nullable(), "log": z.array(z.string()), "log_dropped": z.number(), "outcome": z.string(), "output": z.string().nullable(), "path": z.string(), "shape": z.string().nullable(), "started_at": z.string(), "stderr": outputResponseSchema.nullable(), "stdout": outputResponseSchema.nullable(), "step": z.string(), "values": z.array(renderedResponseSchema), "values_dropped": z.number(), "wait_seconds": z.number().nullable() });

export type TraceEntryResponse = z.infer<typeof traceEntryResponseSchema>;

export const traceResponseSchema = z.object({ "dropped": z.number(), "entries": z.array(traceEntryResponseSchema), "outputs": z.record(z.string(), z.unknown()).nullable().default(null) });

export type TraceResponse = z.infer<typeof traceResponseSchema>;

export const runResponseSchema = z.object({ "arguments": z.array(z.string()), "automation": z.string(), "event": z.string(), "fields": z.record(z.string(), z.string()), "id": z.string(), "outcome": outcomeResponseSchema, "started_at": z.string(), "steps_version": z.string().nullable(), "trace": traceResponseSchema.nullable(), "workflow": z.string().nullable() });

export type RunResponse = z.infer<typeof runResponseSchema>;

export const runSettingsResponseSchema = z.object({ "args": z.array(z.string()), "script": z.string(), "timeout_seconds": z.number() });

export type RunSettingsResponse = z.infer<typeof runSettingsResponseSchema>;

export const whenResponseSchema = z.object({ "cron": z.string().nullable().default(null), "environments": z.array(z.string()).default([]), "event": z.string(), "from": z.array(z.string()).default([]), "from_unknown": z.boolean().default(false), "services": z.array(z.string()).default([]), "to": z.array(z.string()).default([]), "users": z.array(z.string()).default([]), "webhooks": z.array(z.string()).default([]) });

export type WhenResponse = z.infer<typeof whenResponseSchema>;

export const workflowCallResponseSchema = z.object({ "id": z.string(), "inputs": z.record(z.string(), z.unknown()) });

export type WorkflowCallResponse = z.infer<typeof workflowCallResponseSchema>;

export const automationResponseSchema = z.object({ "active_run": runResponseSchema.nullable(), "cooldown_seconds": z.number(), "enabled": z.boolean(), "id": z.string(), "last_run": runResponseSchema.nullable(), "run": runSettingsResponseSchema.nullable(), "tags": z.array(z.string()), "title": z.string(), "when": whenResponseSchema, "workflow": workflowCallResponseSchema.nullable() });

export type AutomationResponse = z.infer<typeof automationResponseSchema>;

export const automationsResponseSchema = z.object({ "automations": z.array(automationResponseSchema) });

export type AutomationsResponse = z.infer<typeof automationsResponseSchema>;

export const schema = automationsResponseSchema;
