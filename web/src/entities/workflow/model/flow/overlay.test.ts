import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { workflowsSchema } from "../schema";
import type { Step } from "../schema";
import { flowOf } from "./build";
import { overlayOf, runPath } from "./overlay";

const lastRun = workflowsSchema.parse(apiSamples.workflows).workflows[0].last_run!;

function entry(path: string, kind: string, outcome: "running" | "succeeded" | "failed", detail = "", iteration: number | null = null) {
  return { path, step: path, label: path, kind, iteration, outcome, started_at: "2026-09-25T03:00:00Z", duration_milliseconds: 10, detail, output: null, shape: null, stdout: null, stderr: null, command: null, budget_reached: false, values: [], log: [], values_dropped: 0, log_dropped: 0, item: null, level: null, wait_seconds: null };
}

describe("workflow run overlay", () => {
  it("the last run of the sample marks its nodes and the branch its if took", () => {
    const overlay = overlayOf(lastRun.trace!.entries);
    expect(overlay.get("steps[0]")?.outcome).toBe("succeeded");
    expect(overlay.get("steps[1]")?.branch).toBe("then");
  });

  it("loop passes add up, a running entry highlights its node, and a failure stays", () => {
    const overlay = overlayOf([
      entry("steps[0].body[0]", "http", "failed", "", 0),
      entry("steps[0].body[0]", "http", "succeeded", "", 1),
      entry("steps[1]", "wait", "running"),
    ]);
    expect(overlay.get("steps[0].body[0]")).toMatchObject({ passes: 2, durationMilliseconds: 20, outcome: "failed", running: false });
    expect(overlay.get("steps[1]")).toMatchObject({ running: true, outcome: "running" });
  });
});

describe("run path", () => {
  const steps: Step[] = [
    { id: "ping", kind: "http", url: "http://x" },
    { id: "check", kind: "if", condition: { left: "a", op: "==", right: "b" }, then: [{ id: "done", kind: "stop", outcome: "succeeded" }], else: [] },
    { id: "retry", kind: "loop", repeat: 2, body: [{ id: "pause", kind: "wait", seconds: 1 }] },
  ];
  const entries = [
    entry("steps[0]", "http", "succeeded"),
    entry("steps[1]", "if", "succeeded", "else"),
    entry("steps[2]", "loop", "succeeded"),
    entry("steps[2].body[0]", "wait", "succeeded", "", 0),
    entry("steps[2].body[0]", "wait", "succeeded", "", 1),
  ];

  it("follows the branch taken, counts the order and returns through a repeated loop", () => {
    const flow = flowOf(steps);
    const overlay = overlayOf(entries);
    const path = runPath(flow, overlay, entries, { active: false, succeeded: true });
    expect([...path.order]).toEqual([
      ["steps[0]", 1],
      ["steps[1]", 2],
      ["steps[2]", 3],
      ["steps[2].body[0]", 4],
    ]);
    expect(path.nodes.has("steps[1]:else:empty")).toBe(true);
    expect(path.nodes.has("steps[1].then[0]")).toBe(false);
    expect(path.edges.has("steps[1]->steps[1]:else:empty")).toBe(true);
    expect(path.edges.has("steps[1]->steps[1].then[0]")).toBe(false);
    expect(path.edges.has("steps[1]:join->steps[2]")).toBe(true);
    expect(path.edges.has("steps[2]:join->steps[2]")).toBe(true);
    expect(path.edges.has("steps[2]:join->end")).toBe(true);
  });

  it("an active run does not reach the end, and a running step closes nothing after it", () => {
    const running = [entry("steps[0]", "http", "succeeded"), entry("steps[1]", "if", "running")];
    const path = runPath(flowOf(steps), overlayOf(running), running, { active: true, succeeded: false });
    expect(path.edges.has("start->steps[0]")).toBe(true);
    expect(path.edges.has("steps[0]->steps[1]")).toBe(true);
    expect(path.nodes.has("steps[1]:join")).toBe(false);
    expect(path.nodes.has("end")).toBe(false);
  });
});
