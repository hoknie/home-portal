import { z } from "zod";

export const receptionResponseSchema = z.object({ "at": z.string(), "status": z.number() });

export type ReceptionResponse = z.infer<typeof receptionResponseSchema>;

export const runSettingsResponseSchema = z.object({ "args": z.array(z.string()), "script": z.string(), "timeout_seconds": z.number() });

export type RunSettingsResponse = z.infer<typeof runSettingsResponseSchema>;

export const workflowCallResponseSchema = z.object({ "id": z.string(), "inputs": z.record(z.string(), z.unknown()) });

export type WorkflowCallResponse = z.infer<typeof workflowCallResponseSchema>;

export const webhookResponseSchema = z.object({ "action": z.string(), "address": z.string(), "enabled": z.boolean(), "id": z.string(), "last_received": receptionResponseSchema.nullable(), "protected": z.boolean(), "run": runSettingsResponseSchema.nullable(), "tags": z.array(z.string()), "title": z.string(), "variables": z.array(z.string()), "workflow": workflowCallResponseSchema.nullable() });

export type WebhookResponse = z.infer<typeof webhookResponseSchema>;

export const createdWebhookResponseSchema = z.object({ "token": z.string().nullable(), "webhook": webhookResponseSchema });

export type CreatedWebhookResponse = z.infer<typeof createdWebhookResponseSchema>;

export const schema = createdWebhookResponseSchema;
