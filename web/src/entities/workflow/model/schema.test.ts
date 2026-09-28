import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { requestOf, workflowCatalogueSchema, workflowRunSchema, workflowsSchema } from "./schema";

describe("workflow schema", () => {
  it("the workflows sample parses with its nested steps, users and last run", () => {
    const parsed = workflowsSchema.parse(apiSamples.workflows);
    const revive = parsed.workflows[0];
    expect(parsed.workflows.map((workflow) => workflow.id)).toEqual(["revive", "note"]);
    expect(revive.steps[1].then?.[0].kind).toBe("loop");
    expect(Array.isArray(revive.steps[1].then?.[0].body)).toBe(true);
    expect(revive.steps[2].branches).toHaveLength(2);
    expect(revive.used_by).toEqual([{ kind: "automation", id: "nas-down", title: "NAS down" }]);
    expect(revive.last_run?.workflow).toBe("revive");
    expect(revive.last_run?.trace?.entries.map((entry) => entry.step)).toEqual(["first_probe", "down"]);
    expect(parsed.workflows[1].enabled).toBe(false);
  });

  it("the catalogue sample lists every kind with its fields and the operators", () => {
    const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);
    expect(catalogue.kinds.map((kind) => kind.name)).toEqual(
      expect.arrayContaining(["if", "loop", "parallel", "workflow", "stop", "set", "wait", "http", "script", "notify", "probe", "status"]),
    );
    const http = catalogue.kinds.find((kind) => kind.name === "http");
    expect(http?.group).toBe("actions");
    expect(http?.fields.find((field) => field.name === "method")?.choices).toContain("POST");
    expect(catalogue.operators.find((operator) => operator.name === "is-empty")?.takes_right).toBe(false);
  });

  it("the run answer parses", () => {
    expect(workflowRunSchema.parse(apiSamples.workflowRun).run_id).toBe("13");
  });

  it("a request keeps the entry and drops the users and runs", () => {
    const revive = workflowsSchema.parse(apiSamples.workflows).workflows[0];
    const body = requestOf(revive);
    expect(Object.keys(body).sort()).toEqual(["description", "enabled", "id", "inputs", "steps", "tags", "timeout_seconds", "title"]);
    expect(body.steps).toBe(revive.steps);
  });
});
