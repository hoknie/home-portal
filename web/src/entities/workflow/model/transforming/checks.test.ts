import { describe, expect, it } from "vitest";

import { scopeAt } from "../scope";
import { checkTemplate, templateNames } from "../suggestions/check";
import type { Step } from "../schema";
import { parseChain } from "./parse";

const steps: Step[] = [
  { id: "ping", kind: "http", url: "http://nas" },
  { id: "bad", kind: "transform", input: "{{steps.ping.json}}", operations: [{ op: "map", to: "{{item.name}}" }] },
  { id: "say", kind: "notify", text: "" },
];

function problems(text: string, at = 2, field = "text") {
  const scope = scopeAt(steps, [{ list: "steps", index: at }], ["service"], field);
  return checkTemplate(text, scope).map((problem) => [problem.reason, problem.params]);
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
