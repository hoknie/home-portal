import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { channelSchema, deliverySchema, notificationsSchema, readyChannels } from "./schema";

describe("notifications", () => {
  it("the samples parse with the rules, the channel state and a test delivery", () => {
    const parsed = notificationsSchema.parse(apiSamples.notifications);
    expect(parsed.rules).toEqual({ states: ["down", "unreadable"], recovered: true });
    expect(parsed.channels[0]).toMatchObject({ name: "telegram", readiness: "ready", queued: 0 });
    expect(parsed.channels[0].settings.secret).toBe("telegram_token");
    expect(readyChannels(parsed)).toEqual(["telegram"]);
    expect(channelSchema.parse(apiSamples.notificationChannelMissing).missing?.field).toBe("notifications.telegram.secret");
    expect(deliverySchema.parse(apiSamples.notificationTest).delivered).toBe(true);
  });
});
