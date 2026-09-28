import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";
import { dictionaries } from "@/shared/i18n";

import { workflowCatalogueSchema } from "./schema";
import { REASONS } from "./suggestions/check";
import { SUGGESTION_GROUPS } from "./suggestions/scope-suggestions";
import { TEMPLATES } from "./templates";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

const SERVER_PHRASES: Record<(typeof REASONS)[number], string> = {
  unknownInput: "is not an input of this workflow",
  unknownVariable: "no earlier step sets the variable",
  unknownStep: "comes earlier",
  outsideLoop: "exists only inside a loop as loop.item and loop.index",
  notAValue: "is not a value a workflow knows",
  notInPortal: "is not a value the portal offers",
  outsideTransform: "exists only inside a transform's filter and map",
  unreadableFilter: "has a filter that cannot be read",
  unknownFilter: "which does not exist",
  filterArguments: "and got",
  filterArgument: "the argument",
  filterType: "takes",
};

type Tree = Record<string, unknown>;

function at(tree: unknown, path: string[]): unknown {
  return path.reduce<unknown>((node, key) => (node !== null && typeof node === "object" ? (node as Tree)[key] : undefined), tree);
}

describe("workflow help", () => {
  it.each(Object.keys(dictionaries))("every kind, field and result of the catalogue has help in %s", (language) => {
    const help = (dictionaries as Record<string, Tree>)[language].workflowHelp;
    const missing: string[] = [];
    const need = (path: string[]) => {
      if (typeof at(help, path) !== "string" || at(help, path) === "") {
        missing.push(path.join("."));
      }
    };
    for (const kind of catalogue.kinds) {
      for (const key of ["name", "description", "example"]) {
        need(["kinds", kind.name, key]);
      }
      for (const field of kind.fields) {
        need(["fields", kind.name, field.name, "label"]);
        need(["fields", kind.name, field.name, "help"]);
      }
      for (const result of kind.results) {
        need(["results", kind.name, result]);
      }
      need(["groups", kind.group, "name"]);
    }
    for (const filter of catalogue.filters) {
      need(["filters", filter.name, "description"]);
      need(["filters", filter.name, "example"]);
    }
    for (const operation of catalogue.operations) {
      need(["operations", operation.name, "name"]);
      need(["operations", operation.name, "description"]);
    }
    REASONS.forEach((reason) => need(["reasons", reason]));
    SUGGESTION_GROUPS.forEach((group) => need(["suggestions", "groups", group]));
    for (const template of TEMPLATES) {
      need(["templates", template.name, "title"]);
      need(["templates", template.name, "description"]);
    }
    expect(missing).toEqual([]);
  });

  it("the reasons in English say what the server says", () => {
    const reasons = dictionaries.en.workflowHelp.reasons as Record<string, string>;
    for (const reason of REASONS) {
      expect(reasons[reason]).toContain(SERVER_PHRASES[reason]);
    }
  });
});
