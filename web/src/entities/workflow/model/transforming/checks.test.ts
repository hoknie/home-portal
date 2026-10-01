import { describe, expect, it } from "vitest";

import { scopeAt } from "../scope";
import { checkTemplate, templateNames } from "../suggestions/check";
import { type Step, workflowCatalogueSchema } from "../schema";
import { apiSamples } from "@/shared/api";

import { parseChain } from "./parse";
import { type PreviewQuestion, askPreview } from "./previews";
import { chainProblem } from "./types";

const filters = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue).filters;

const steps: Step[] = [
  { id: "ping", kind: "http", url: "http://nas" },
  { id: "bad", kind: "transform", input: "{{steps.ping.json}}", operations: [{ op: "map", to: "{{item.name}}" }] },
  { id: "say", kind: "notify", text: "" },
];

function problems(text: string, at = 2, field = "text") {
  const scope = scopeAt(steps, [{ list: "steps", index: at }], ["service"], field);
  return checkTemplate(text, scope, null, null, filters).map((problem) => [problem.reason, problem.params]);
}

describe("filters in templates", () => {
  it("a name keeps its filters, with spaces around the bars and quoted commas", () => {
    const [found] = templateNames("x {{ steps.ping.json | pluck('a,b') | join(\", \") }} y");
    expect(found).toMatchObject({ name: "steps.ping.json", valid: true, end: 51 });
    expect(found.filters).toEqual([
      { name: "pluck", arguments: ["a,b"] },
      { name: "join", arguments: [", "] },
    ]);
    expect(parseChain("join(").error).toBe("join(");
  });

  it("an unknown filter, wrong arguments and a certain type mismatch are problems", () => {
    expect(problems("{{inputs.service | shout}}")).toEqual([["unknownFilter", { filter: "shout" }]]);
    expect(problems("{{inputs.service | split}}")).toEqual([["filterArguments", { filter: "split", wanted: "1", got: "0" }]]);
    expect(problems("{{inputs.service | split(3)}}")).toEqual([["filterArgument", { filter: "split", argument: "separator" }]]);
    expect(problems("{{steps.ping.status | join(', ')}}")).toEqual([["filterType", { filter: "join", takes: "list", got: "number" }]]);
    expect(problems("{{steps.ping.json | pluck('a') | join(', ') | upper}} {{inputs.service | join(}}")).toEqual([["unreadableFilter", { filter: "join(" }]]);
  });

  it("item and index are known only inside a transform's operations", () => {
    expect(problems("{{item.name}}")).toEqual([["outsideTransform", { name: "item.name" }]]);
    expect(problems("{{item.name | upper}} {{index}}", 1, "operations[0].to")).toEqual([]);
    expect(problems("{{item}}", 1, "input")).toEqual([["outsideTransform", { name: "item" }]]);
  });
});

describe("variables in filter and operation arguments", () => {
  it("a bare name is a name argument, a quoted text a literal, and a word outside the namespaces unreadable", () => {
    expect(parseChain("get(loop.item)").filters).toEqual([{ name: "get", arguments: [null], names: [{ position: 0, name: "loop.item" }] }]);
    expect(parseChain('get("loop.item")').filters).toEqual([{ name: "get", arguments: ["loop.item"] }]);
    expect(parseChain("get(name)").error).toBe("get(name)");
  });

  it("a name the editor knows is sent with the preview, and an unknown one says it is known only when the step runs", () => {
    const asked: PreviewQuestion[] = [];
    const cache = (question: PreviewQuestion) => {
      asked.push(question);
      return { state: "ready" as const, answer: { input: { value: { nas: 2 }, error: null }, steps: [{ value: 2, error: null }], examples: [] } };
    };
    const question = { value: { nas: 2 }, filters: "", operations: [{ op: "get", args: ["{{vars.which}}"] }], examples: [] };
    expect(askPreview(cache, question, () => null).steps[0].unknown).toBe("vars.which");
    expect(askPreview(cache, question, (name) => (name === "vars.which" ? { value: "nas" } : null)).steps[0].value).toBe(2);
    expect(asked.at(-1)?.names).toEqual({ "vars.which": "nas" });
  });

  it("a templated argument skips the literal type check", () => {
    expect(chainProblem("text", [{ name: "slice", arguments: ["{{inputs.start}}"] }], filters)).toBeNull();
    expect(chainProblem("text", [{ name: "slice", arguments: [null], names: [{ position: 0, name: "inputs.start" }] }], filters)).toBeNull();
    expect(chainProblem("text", [{ name: "slice", arguments: ["x"] }], filters)?.reason).toBe("filterArgument");
  });
});

it("an argument name out of scope is marked with the reason a name out of scope gets", () => {
  expect(problems("{{steps.ping.json | get(loop.item)}}").map(([reason]) => reason)).toEqual(["outsideLoop"]);
});
