import { z } from "zod";

export const deliveryResponseSchema = z.object({ "at": z.string(), "channel": z.string(), "delivered": z.boolean(), "error": z.string().nullable() });

export type DeliveryResponse = z.infer<typeof deliveryResponseSchema>;

export const lastErrorResponseSchema = z.object({ "at": z.string(), "message": z.string() });

export type LastErrorResponse = z.infer<typeof lastErrorResponseSchema>;

export const missingResponseSchema = z.object({ "field": z.string(), "message": z.string() });

export type MissingResponse = z.infer<typeof missingResponseSchema>;

export const channelResponseSchema = z.object({ "dropped": z.number(), "last_delivery": deliveryResponseSchema.nullable(), "last_error": lastErrorResponseSchema.nullable(), "missing": missingResponseSchema.nullable(), "name": z.string(), "queued": z.number(), "readiness": z.string(), "settings": z.record(z.string(), z.unknown()) });

export type ChannelResponse = z.infer<typeof channelResponseSchema>;

export const rulesResponseSchema = z.object({ "recovered": z.boolean(), "states": z.array(z.string()) });

export type RulesResponse = z.infer<typeof rulesResponseSchema>;

export const notificationsResponseSchema = z.object({ "channels": z.array(channelResponseSchema), "enabled": z.boolean(), "rules": rulesResponseSchema });

export type NotificationsResponse = z.infer<typeof notificationsResponseSchema>;

export const schema = notificationsResponseSchema;
