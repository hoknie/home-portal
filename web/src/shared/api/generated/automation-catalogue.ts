import { z } from "zod";

export const choiceResponseSchema = z.object({ "id": z.string(), "name": z.string() });

export type ChoiceResponse = z.infer<typeof choiceResponseSchema>;

export const webhookChoiceResponseSchema = z.object({ "action": z.string(), "id": z.string(), "name": z.string(), "variables": z.array(z.string()) });

export type WebhookChoiceResponse = z.infer<typeof webhookChoiceResponseSchema>;

export const choicesResponseSchema = z.object({ "environments": z.array(z.string()), "services": z.array(choiceResponseSchema), "tags": z.array(z.string()), "users": z.array(z.string()), "webhooks": z.array(webhookChoiceResponseSchema) });

export type ChoicesResponse = z.infer<typeof choicesResponseSchema>;

export const fieldResponseSchema = z.object({ "name": z.string(), "sample": z.string() });

export type FieldResponse = z.infer<typeof fieldResponseSchema>;

export const eventResponseSchema = z.object({ "fields": z.array(fieldResponseSchema), "filters": z.array(z.string()), "name": z.string() });

export type EventResponse = z.infer<typeof eventResponseSchema>;

export const catalogueResponseSchema = z.object({ "choices": choicesResponseSchema, "events": z.array(eventResponseSchema), "states": z.array(z.string()) });

export type CatalogueResponse = z.infer<typeof catalogueResponseSchema>;

export const schema = catalogueResponseSchema;
