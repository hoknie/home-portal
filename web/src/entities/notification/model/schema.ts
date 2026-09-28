import { z } from "zod";

export const READINESS = ["ready", "disabled", "missing"] as const;

export const deliverySchema = z.object({
  channel: z.string(),
  at: z.string(),
  delivered: z.boolean(),
  error: z.string().nullable(),
});

export type Delivery = z.infer<typeof deliverySchema>;

export const channelSchema = z.object({
  name: z.string(),
  readiness: z.enum(READINESS),
  missing: z.object({ field: z.string(), message: z.string() }).nullable(),
  settings: z.record(z.string(), z.unknown()),
  last_delivery: deliverySchema.nullable(),
  last_error: z.object({ at: z.string(), message: z.string() }).nullable(),
  queued: z.number(),
  dropped: z.number(),
});

export type NotificationChannel = z.infer<typeof channelSchema>;

export const notificationsSchema = z.object({
  enabled: z.boolean(),
  rules: z.object({ states: z.array(z.string()), recovered: z.boolean() }),
  channels: z.array(channelSchema),
});

export type Notifications = z.infer<typeof notificationsSchema>;

export type Rules = Notifications["rules"];

export function readyChannels(notifications: Notifications | undefined) {
  return (notifications?.channels ?? []).filter((channel) => channel.readiness === "ready").map((channel) => channel.name);
}
