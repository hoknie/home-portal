import { expect, it } from "vitest";

import { historySchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";

import { durationParts, percent, slotsOf } from "./history";

const history = historySchema.parse(apiSamples.history);

it("a day is 24 hourly slots, each carrying the worst state probed in it, and empty hours stay empty", () => {
  const now = Date.parse(history.to);
  const slots = slotsOf(history.points, "24h", now);
  expect(slots).toHaveLength(24);
  expect(slots[23].state).toBe("down");
  expect(slots.slice(0, 23).every((slot) => slot.state === null)).toBe(true);
});

it("durations and percentages read the way a person says them", () => {
  expect(durationParts(90 * 60_000 + 1_440 * 60_000)).toEqual({ days: 1, hours: 1, minutes: 30 });
  expect(percent(0.99987)).toBe(99.9);
  expect(percent(null)).toBeNull();
});
