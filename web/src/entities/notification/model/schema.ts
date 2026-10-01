import { z } from "zod";

import { generated } from "@/shared/api";

export const READINESS = ["ready", "disabled", "missing"] as const;

const served = generated.notifications;

export const deliverySchema = served.deliveryResponseSchema;

export type Delivery = z.infer<typeof deliverySchema>;

export const channelSchema = served.channelResponseSchema.extend({ readiness: z.enum(READINESS) });

export type NotificationChannel = z.infer<typeof channelSchema>;

export const notificationsSchema = served.notificationsResponseSchema.extend({ channels: z.array(channelSchema) });

export type Notifications = z.infer<typeof notificationsSchema>;

export type Rules = Notifications["rules"];

export function readyChannels(notifications: Notifications | undefined) {
  return (notifications?.channels ?? []).filter((channel) => channel.readiness === "ready").map((channel) => channel.name);
}
