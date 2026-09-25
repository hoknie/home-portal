import { describe, expect, it } from "vitest";

import { labelsFor, timeTicks, valueTicks } from "./ticks";

const HOUR = 3_600_000;
const DAY = 24 * HOUR;
const TO = new Date(2026, 8, 23, 14, 37).getTime();

describe.each([
  ["24h", DAY],
  ["7d", 7 * DAY],
  ["30d", 30 * DAY],
])("the %s range", (_, span) => {
  it.each([320, 390, 768, 1280])("never has more labels than fit in %i px, and falls on round local times", (width) => {
    const ticks = timeTicks(TO - span, TO, width);
    expect(ticks.length).toBeGreaterThanOrEqual(2);
    expect(ticks.length).toBeLessThanOrEqual(labelsFor(width));
    for (const tick of ticks) {
      const date = new Date(tick.at);
      expect(date.getMinutes() % 15).toBe(0);
      expect(date.getSeconds()).toBe(0);
      if (span > DAY) {
        expect(date.getHours()).toBe(0);
      }
      expect(tick.position).toBeGreaterThanOrEqual(0);
      expect(tick.position).toBeLessThanOrEqual(1);
    }
  });
});

it("places a tick by its share of the range", () => {
  const from = new Date(2026, 8, 23, 0, 0).getTime();
  const ticks = timeTicks(from, from + DAY, 1280);
  const noon = ticks.find((tick) => new Date(tick.at).getHours() === 12);
  expect(noon?.position).toBeCloseTo(0.5);
});

it("an empty range has no ticks", () => {
  expect(timeTicks(TO, TO, 800)).toEqual([]);
});

it.each([
  [0, [0, 1, 2]],
  [7, [0, 5, 10]],
  [430, [0, 200, 400, 600]],
  [12_000, [0, 5000, 10_000, 15_000]],
])("values up to %i ms get round ticks from zero", (highest, expected) => {
  const { ticks, top } = valueTicks(highest);
  expect(ticks).toEqual(expected);
  expect(top).toBeGreaterThanOrEqual(highest);
  expect(ticks.length).toBeGreaterThanOrEqual(3);
  expect(ticks.length).toBeLessThanOrEqual(6);
});
