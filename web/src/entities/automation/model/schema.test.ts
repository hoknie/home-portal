import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";
import { TRACE_ENTRY_BLANKS } from "@/shared/lib/testing";

import {
  automationsSchema,
  catalogueSchema,
  eventNameSchema,
  isActive,
  messageKeyOf,
  outcomeSchema,
  queuedSchema,
  runsSchema,
  scheduleSchema,
  scriptsSchema,
  traceSchema,
} from "./schema";
import { exitCodeOf, scriptLogOf } from "./script-log";

describe("automation", () => {
  it("the automations sample parses with its last run", () => {
    const parsed = automationsSchema.parse(apiSamples.automations);
    expect(parsed.automations.map((automation) => automation.id)).toEqual(["restart-media", "backup", "motion-alarm"]);
    expect(parsed.automations[0].when).toMatchObject({ event: "service.status-changed", services: ["jellyfin"], to: ["down", "unreadable"] });
    expect(parsed.automations[0].last_run?.outcome).toMatchObject({ result: "skipped", reason: "cooldown", count: 9 });
    expect(parsed.automations[1].when.cron).toBe("0 3 * * *");
    expect(parsed.automations[0].active_run).toBeNull();
    expect(parsed.automations[1].active_run?.outcome.result).toBe("running");
  });

  it("the runs sample parses with its outputs", () => {
    const parsed = runsSchema.parse(apiSamples.automationRuns);
    expect(parsed.runs.map((run) => run.outcome.result)).toEqual(["running", "stopped", "failed", "skipped", "succeeded", "succeeded"]);
    expect(parsed.runs.map(isActive)).toEqual([true, false, false, false, false, false]);
    expect(parsed.runs[1].outcome.reason).toBe("stopped by admin");
    expect(parsed.runs[2].outcome.stderr.tail).toBe("disk full\n");
    expect(parsed.runs[4].outcome.stdout.truncated).toBe(true);
  });

  it("the catalogue, scripts, schedule and queued samples parse", () => {
    const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
    expect(catalogue.events.map((event) => event.name)).toContain("service.status-changed");
    expect(catalogue.choices.environments).toContain("internet");
    expect(scriptsSchema.parse(apiSamples.automationScripts).scripts.find((script) => script.path === "open.sh")).toMatchObject({ runnable: false });
    expect(scheduleSchema.parse(apiSamples.automationSchedule).times).toHaveLength(5);
    expect(queuedSchema.parse(apiSamples.automationQueued).run_id).toBe("42");
  });

  it("a script entry of the sample parses with its streams and command", () => {
    const sample = apiSamples.workflows as { workflows: { last_run: { trace: unknown } }[] };
    const entries = traceSchema.parse(sample.workflows[0].last_run.trace).entries;
    const script = entries.find((entry) => entry.kind === "script");
    expect(script).toMatchObject({ stdout: { tail: "stopping jellyfin\nstarted\n", bytes: 25 }, command: ["restart.sh", "jellyfin"], budget_reached: false, output: null });
    expect(entries.find((entry) => entry.kind === "if")).toMatchObject({ stdout: null, stderr: null, command: null });
  });

  it("an entry from before separate streams parses and keeps its single output", () => {
    const old = { ...TRACE_ENTRY_BLANKS, path: "steps[0]", step: "run", label: "run", kind: "script", iteration: null, outcome: "failed", started_at: "2026-01-01T00:00:00Z", duration_milliseconds: 3, detail: "restart.sh exited 1", output: "progress\ncontainer not found\n" };
    const [entry] = traceSchema.parse({ entries: [old], dropped: 0 }).entries;
    expect(entry).toMatchObject({ stdout: null, stderr: null, command: null, budget_reached: false });
    expect(scriptLogOf(entry)).toEqual({ kind: "merged", output: { tail: "progress\ncontainer not found\n", bytes: 29, truncated: false } });
    expect(exitCodeOf(entry)).toBe(1);
    expect(exitCodeOf({ ...entry, detail: "restart.sh exited 1: container not found" })).toBe(1);
    expect(exitCodeOf({ ...entry, detail: "restart.sh timed out after 60 s" })).toBeNull();
  });

  it("an event or outcome the interface does not know becomes unknown", () => {
    expect(eventNameSchema.parse("service.exploded")).toBe("unknown");
    expect(outcomeSchema.parse("vanished")).toBe("unknown");
  });

  it("a name becomes a message key without dots or dashes", () => {
    expect(messageKeyOf("service.status-changed")).toBe("service_status_changed");
    expect(messageKeyOf("sign_in.reason")).toBe("sign_in_reason");
  });
});
