import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { workflowCatalogueSchema } from "../schema";
import { FILTERS } from "./filters";
import { type Operation, transformSample } from "./operations";

type Case = { name: string; input: unknown; operations: Operation[]; outcome: { value: unknown } | { error: string } };

const cases = (apiSamples.transforms as { cases: Case[] }).cases;

describe("the transform fixtures shared with the server", () => {
  it.each(cases.map((entry) => [entry.name, entry] as const))("%s gives what the server gives", (_, entry) => {
    expect(transformSample(entry.input, entry.operations)).toEqual(entry.outcome);
  });

  it("the filters this evaluator knows are the catalogue's, with the same types and arguments", () => {
    const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);
    const served = catalogue.filters.map((filter) => [
      filter.name,
      { accepts: filter.accepts, gives: filter.gives, element: filter.element, arguments: filter.arguments.map(({ name, type, required }) => ({ name, type, required })) },
    ]);
    expect(Object.fromEntries(served)).toEqual(FILTERS);
  });
});
