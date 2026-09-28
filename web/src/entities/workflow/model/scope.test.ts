import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { chipsOf, scopeAt } from "./scope";
import { type Step, workflowCatalogueSchema } from "./schema";
import { parsePath } from "./tree";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

const steps: Step[] = [
  { id: "list", kind: "http", method: "GET", url: "http://nas.lan/api/disks" },
  {
    id: "each",
    kind: "loop",
    for_each: "{{steps.list.json}}",
    body: [
      { id: "remember", kind: "set", variable: "disk", value: "{{loop.item}}" },
      { id: "tell", kind: "notify", text: "" },
    ],
  },
  { id: "after", kind: "notify", text: "" },
];

describe("workflow scope", () => {
  it("chips inside a for_each loop after an http step include loop values and the step's json, not later steps", () => {
    const scope = scopeAt(steps, parsePath("steps[1].body[1]").path, ["service"]);
    const chips = chipsOf(scope, catalogue);
    expect(chips).toEqual(expect.arrayContaining(["loop.item", "loop.index", "steps.list.json", "inputs.service", "vars.disk", "steps.each.iterations"]));
    expect(chips.some((chip) => chip.startsWith("steps.after."))).toBe(false);
    expect(chips.some((chip) => chip.startsWith("steps.tell."))).toBe(false);
  });

  it("a step does not see itself, and outside the loop there is no loop value", () => {
    const scope = scopeAt(steps, parsePath("steps[2]").path, []);
    expect(scope.inLoop).toBe(false);
    expect(scope.steps.map((step) => step.id)).toEqual(["list", "each", "remember", "tell"]);
    expect(scopeAt(steps, parsePath("steps[0]").path, []).steps).toEqual([]);
  });

  it("a loop's own while condition sees loop values, but its for_each does not", () => {
    expect(scopeAt(steps, parsePath("steps[1]").path, [], "while.left").inLoop).toBe(true);
    expect(scopeAt(steps, parsePath("steps[1]").path, [], "for_each").inLoop).toBe(false);
  });

  it("event fields become chips", () => {
    expect(chipsOf(scopeAt(steps, parsePath("steps[0]").path, []), catalogue, ["service.id"])).toEqual(["event.service.id"]);
  });
});
