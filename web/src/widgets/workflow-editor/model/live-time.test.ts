import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { type TraceEntry, traceSchema } from "@/entities/automation";

import { elapsedOf, timeLeftOf, useNow } from "./live-time";

function entry(extra: Partial<TraceEntry>): TraceEntry {
  const [parsed] = traceSchema.parse({ entries: [{ path: "steps[0]", step: "nap", label: "nap", kind: "wait", iteration: null, outcome: "running", started_at: "2026-09-29T10:00:00Z", duration_milliseconds: 12_000, detail: "", output: null }], dropped: 0 }).entries;
  return { ...parsed, ...extra };
}

afterEach(() => {
  vi.useRealTimers();
});

describe("live time", () => {
  it("an old entry without wait_seconds parses, and has no time left", () => {
    expect(entry({}).wait_seconds).toBeNull();
    expect(timeLeftOf(entry({}), 0, 0)).toBeNull();
  });

  it("elapsed time grows from the answer's duration by the client's own clock", () => {
    expect(elapsedOf(entry({}), 5_000, 5_000)).toBe(12_000);
    expect(elapsedOf(entry({}), 5_000, 8_000)).toBe(15_000);
    expect(elapsedOf(entry({ outcome: "succeeded", duration_milliseconds: 20_000 }), 5_000, 99_000)).toBe(20_000);
  });

  it("time left counts down and stops at zero", () => {
    const nap = entry({ wait_seconds: 60 });
    expect(timeLeftOf(nap, 0, 0)).toBe(48_000);
    expect(timeLeftOf(nap, 0, 1_000)).toBe(47_000);
    expect(timeLeftOf(nap, 0, 500_000)).toBe(0);
  });

  it("a client clock far from the server's changes neither", () => {
    const nap = entry({ wait_seconds: 60 });
    const skewed = 9_000_000_000_000;
    expect(elapsedOf(nap, skewed, skewed + 2_000)).toBe(14_000);
    expect(timeLeftOf(nap, skewed, skewed + 2_000)).toBe(46_000);
  });

  it("the clock ticks every second only while asked", () => {
    vi.useFakeTimers();
    vi.setSystemTime(10_000);
    const { result, rerender } = renderHook(({ ticking }) => useNow(ticking), { initialProps: { ticking: true } });
    act(() => {
      vi.advanceTimersByTime(2_000);
    });
    expect(result.current).toBe(12_000);
    rerender({ ticking: false });
    act(() => {
      vi.advanceTimersByTime(5_000);
    });
    expect(result.current).toBe(12_000);
  });
});
