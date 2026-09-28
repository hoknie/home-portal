import { describe, expect, it } from "vitest";

import { workflowCatalogueSchema, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";

import type { Draft } from "../draft";
import { type ProblemContext, blocking, problemsFor } from "./problems";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);
const note = workflowsSchema.parse(apiSamples.workflows).workflows[1];

const draft: Draft = {
  id: "check",
  title: "Check",
  enabled: true,
  description: null,
  tags: [],
  timeout_seconds: 300,
  inputs: [],
  steps: [
    { id: "ping", kind: "http", url: "http://nas.lan", headers: { Authorization: "Bearer {{secrets.calendar_password}}" } },
    { id: "tell", kind: "notify", text: "{{steps.ping.json.missing}} {{steps.later.status}}" },
    { id: "wait", kind: "wait", seconds: 0 },
  ],
};

function context(extra: Partial<ProblemContext> = {}): ProblemContext {
  return {
    draft,
    catalogue,
    taken: [],
    server: {},
    secrets: [{ name: "calendar_password", set: false }],
    lastRun: {
      entries: [{ path: "steps[0]", step: "ping", label: "ping", kind: "http", iteration: null, outcome: "succeeded", started_at: "", duration_milliseconds: 1, detail: "", output: '{"state":"up"}', shape: null, values: [], log: [], values_dropped: 0, log_dropped: 0, item: null, level: null }],
      dropped: 0,
    },
    workflow: { ...note, used_by: [] },
    ...extra,
  };
}

describe("editor problems", () => {
  it("merges structure, scope, server and warnings in flow order", () => {
    const problems = problemsFor(context({ server: { "steps[0].url": "must be reachable" } }));
    expect(problems.map((problem) => [problem.at, problem.severity, problem.key ?? problem.text])).toEqual([
      ["", "warning", "workflowEditor.problems.notStarted"],
      ["steps[0].headers.Authorization", "warning", "workflowEditor.problems.secretUnset"],
      ["steps[0].url", "error", "must be reachable"],
      ["steps[1].text", "error", "workflowHelp.reasons.unknownStep"],
      ["steps[1].text", "warning", "workflowEditor.problems.unseenKey"],
      ["steps[2].seconds", "error", "workflowEditor.problems.between"],
    ]);
  });

  it("warnings never block saving", () => {
    const clean: Draft = { ...draft, steps: [{ id: "ping", kind: "http", url: "http://nas.lan", headers: { Authorization: "{{secrets.calendar_password}}" } }] };
    const problems = problemsFor(context({ draft: clean }));
    expect(problems.length).toBeGreaterThan(0);
    expect(blocking(problems)).toEqual([]);
  });
});
