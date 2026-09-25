import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { historySchema } from "./history";

it("the history sample parses: points, uptime for three ranges and transitions", () => {
  const history = historySchema.parse(apiSamples.history);
  expect(history.range).toBe("24h");
  expect(history.uptime.map((uptime) => uptime.range)).toEqual(["24h", "7d", "30d"]);
  expect(history.uptime[0].ratio).toBeCloseTo(0.8);
  expect(history.points).toHaveLength(6);
  expect(history.points[3]).toMatchObject({ state: "down", average: null });
  expect(history.transitions.map((transition) => transition.to)).toEqual(["up", "down", "up"]);
});

it("a state the interface does not know reads as unknown", () => {
  const future = { ...apiSamples.history, points: [{ ...apiSamples.history.points[0], state: "sleeping" }] };
  expect(historySchema.parse(future).points[0].state).toBe("unknown");
});
