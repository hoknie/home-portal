import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { ACTIVE_RUNS_MILLISECONDS, LIVE_RUNS_MILLISECONDS, runRefreshInterval, runsRefreshInterval } from "./queries";
import { runsSchema } from "./schema";

const [running, stopped, failed] = runsSchema.parse(apiSamples.automationRuns).runs;

describe("refresh intervals", () => {
  it("the journal refreshes every second while a run is active, every five seconds otherwise, and never when live is off", () => {
    expect(runsRefreshInterval([running, failed], true)).toBe(ACTIVE_RUNS_MILLISECONDS);
    expect(runsRefreshInterval([stopped, failed], true)).toBe(LIVE_RUNS_MILLISECONDS);
    expect(runsRefreshInterval(undefined, true)).toBe(LIVE_RUNS_MILLISECONDS);
    expect(runsRefreshInterval([running], false)).toBe(false);
  });

  it("an open run is followed every second until it finishes", () => {
    expect(runRefreshInterval(undefined)).toBe(ACTIVE_RUNS_MILLISECONDS);
    expect(runRefreshInterval(running)).toBe(ACTIVE_RUNS_MILLISECONDS);
    expect(runRefreshInterval(stopped)).toBe(false);
  });
});
