import { describe, expect, it } from "vitest";

import { LOOP_EXITS, type Step, chosenOf, newStep, workflowCatalogueSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";

import { problemsOf } from "./validation";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);
const defaults = apiSamples.stepDefaults as Record<string, Record<string, unknown>>;
const SKIPPED = new Set(["id", "kind", "label"]);

function filled(kindName: string): Step {
  const kind = catalogue.kinds.find((entry) => entry.name === kindName)!;
  const fixture = defaults[kindName];
  const step: Record<string, unknown> = { ...newStep(kind, new Set()), id: fixture.id };
  for (const [key, value] of Object.entries(fixture)) {
    const field = kind.fields.find((entry) => entry.name === key);
    if (!SKIPPED.has(key) && (field?.required || key in step)) {
      step[key] = value;
    }
  }
  return step as Step;
}

describe("every kind from the palette", () => {
  it("the fixture covers every kind of the catalogue", () => {
    expect(Object.keys(defaults).sort()).toEqual(catalogue.kinds.map((kind) => kind.name).sort());
  });

  it.each(catalogue.kinds.map((kind) => [kind.name] as const))("a new %s with its required fields filled has no problem and the server's keys", (name) => {
    const step = filled(name);
    const kind = catalogue.kinds.find((entry) => entry.name === name)!;
    const keys = (value: Record<string, unknown>) => Object.keys(value).filter((key) => !SKIPPED.has(key)).sort();
    expect(keys(step as Record<string, unknown>)).toEqual(keys(defaults[name]));
    for (const group of kind.exclusive) {
      expect(group.filter((field) => (step as Record<string, unknown>)[field] !== undefined)).toEqual([chosenOf(step, group)]);
    }
    const placed = (LOOP_EXITS as readonly string[]).includes(name) ? { id: "wrap", kind: "loop", repeat: 1, body: [step] } : step;
    const draft = { id: "all", title: "All", enabled: true, description: null, tags: [], timeout_seconds: 300, inputs: [], outputs: [], steps: [placed] };
    const problems = Object.keys(problemsOf(draft, catalogue, [])).filter((at) => at.startsWith("steps[0]"));
    expect(problems).toEqual([]);
  });
});
