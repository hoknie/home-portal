import { expect, it } from "vitest";

import { durationParts, percent } from "./history";

it("durations and percentages read the way a person says them", () => {
  expect(durationParts(90 * 60_000 + 1_440 * 60_000)).toEqual({ days: 1, hours: 1, minutes: 30 });
  expect(percent(0.99987)).toBe(99.9);
  expect(percent(null)).toBeNull();
});
