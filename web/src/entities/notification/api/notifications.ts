import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type Rules, deliverySchema, notificationsSchema } from "../model/schema";

export function fetchNotifications() {
  return request(api.notifications, { schema: notificationsSchema });
}

export function changeRules(rules: Rules, revision: string | null) {
  return request(api.notifications, { method: "PUT", body: rules, revision, schema: notificationsSchema });
}

export function changeChannel(name: string, settings: Record<string, unknown>, revision: string | null) {
  return request(api.notificationChannel(name), { method: "PUT", body: settings, revision, schema: notificationsSchema });
}

export async function sendTest(channel: string) {
  return (await request(api.notificationTest, { method: "POST", body: { channel }, schema: deliverySchema })).data;
}
