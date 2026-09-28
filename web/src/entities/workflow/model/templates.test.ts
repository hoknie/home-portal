import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { workflowCatalogueSchema } from "./schema";
import { scopeAt } from "./scope";
import { checkTemplate } from "./suggestions/check";
import { TEMPLATES, templateNamed } from "./templates";
import { everyStep, pathText } from "./tree";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

describe("starter templates", () => {
  it.each(TEMPLATES.filter((template) => template.name !== "empty").map((template) => [template.name, template]))("%s has valid steps and no unknown names", (_, template) => {
    const ids = new Set<string>();
    everyStep(template.draft.steps, (step, path) => {
      const kind = catalogue.kinds.find((entry) => entry.name === step.kind);
      expect(kind, `${pathText(path)} kind`).toBeDefined();
      expect(ids.has(step.id), `${step.id} unique`).toBe(false);
      ids.add(step.id);
      for (const field of kind!.fields.filter((entry) => entry.required && entry.type !== "steps")) {
        expect((step as Record<string, unknown>)[field.name], `${pathText(path)}.${field.name}`).toBeDefined();
      }
      for (const [field, value] of Object.entries(step)) {
        if (typeof value === "string") {
          expect(checkTemplate(value, scopeAt(template.draft.steps, path, template.draft.inputs, field)), `${pathText(path)}.${field}`).toEqual([]);
        }
      }
    });
    expect(template.draft.id).toMatch(/^[a-z][a-z0-9-]*$/);
  });

  it("a template is found by its name", () => {
    expect(templateNamed("retry")?.draft.steps[0].kind).toBe("loop");
    expect(templateNamed("nope")).toBeUndefined();
  });
});
