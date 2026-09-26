import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

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
} from "./schema";

describe("automation", () => {
  it("the automations sample parses with its last run", () => {
    const parsed = automationsSchema.parse(apiSamples.automations);
    expect(parsed.automations.map((automation) => automation.id)).toEqual(["restart-media", "backup"]);
    expect(parsed.automations[0].when).toMatchObject({ event: "service.status-changed", services: ["jellyfin"], to: ["down", "unreadable"] });
    expect(parsed.automations[0].last_run?.outcome).toMatchObject({ result: "skipped", reason: "cooldown", count: 9 });
    expect(parsed.automations[1].when.cron).toBe("0 3 * * *");
    expect(parsed.automations[0].active_run).toBeNull();
    expect(parsed.automations[1].active_run?.outcome.result).toBe("running");
  });

  it("the runs sample parses with its outputs", () => {
    const parsed = runsSchema.parse(apiSamples.automationRuns);
    expect(parsed.runs.map((run) => run.outcome.result)).toEqual(["running", "stopped", "failed", "skipped", "succeeded"]);
    expect(parsed.runs.map(isActive)).toEqual([true, false, false, false, false]);
    expect(parsed.runs[1].outcome.reason).toBe("stopped by admin");
    expect(parsed.runs[2].outcome.stderr.tail).toBe("disk full\n");
    expect(parsed.runs[4].outcome.stdout.truncated).toBe(true);
  });

  it("the catalogue, scripts, schedule and queued samples parse", () => {
    const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
    expect(catalogue.events.map((event) => event.name)).toContain("service.status-changed");
    expect(catalogue.choices.environments).toContain("internet");
    expect(scriptsSchema.parse(apiSamples.automationScripts).scripts[1]).toMatchObject({ runnable: false });
    expect(scheduleSchema.parse(apiSamples.automationSchedule).times).toHaveLength(5);
    expect(queuedSchema.parse(apiSamples.automationQueued).run_id).toBe("42");
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
