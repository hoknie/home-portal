import { expect, it } from "vitest";

import { workflowCatalogueSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";

import type { Draft } from "../draft";
import { problemsOf } from "./validation";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

function draftWith(id: string): Draft {
  return { id, title: "Check", enabled: true, description: null, tags: [], timeout_seconds: 300, inputs: [], outputs: [], steps: [{ id: "pause", kind: "wait", seconds: 1 }] };
}

it.each(["new", "edit", "catalogue", "runs"])("the id %s names an address of the portal and is reported before saving", (id) => {
  expect(problemsOf(draftWith(id), catalogue, []).id).toBe("workflowEditor.problems.takenId");
});

it("an ordinary id is not reserved", () => {
  expect(problemsOf(draftWith("news"), catalogue, []).id).toBeUndefined();
});
