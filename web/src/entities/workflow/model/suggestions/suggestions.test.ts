import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { type Step, workflowCatalogueSchema, workflowsSchema } from "../schema";
import { scopeAt } from "../scope";
import { parsePath } from "../tree";
import { checkTemplate } from "./check";
import { jsonKeys } from "./json-keys";
import { type SuggestionContext, suggestionsAt } from "./scope-suggestions";
import { knownValues } from "./values";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);
const workflows = workflowsSchema.parse(apiSamples.workflows).workflows;

const steps: Step[] = [
  { id: "remember", kind: "set", variable: "disk", value: "sda" },
  { id: "list", kind: "http", url: "http://nas.lan/disks" },
  {
    id: "each",
    kind: "loop",
    for_each: "{{steps.list.json}}",
    while: { left: "{{loop.index}}", op: "<", right: "3" },
    body: [
      { id: "tell", kind: "notify", text: "" },
      { id: "both", kind: "parallel", branches: [[{ id: "a", kind: "wait", seconds: 1 }], [{ id: "b", kind: "wait", seconds: 1 }]] },
    ],
  },
  { id: "call", kind: "workflow", workflow: "note" },
  { id: "after", kind: "notify", text: "" },
];

function context(path: string, extra: Partial<SuggestionContext> = {}): SuggestionContext {
  return {
    steps,
    inputs: ["service"],
    path: parsePath(path).path,
    field: parsePath(path).field,
    catalogue,
    workflows,
    eventFields: [{ name: "service.id", sample: "jellyfin" }],
    secrets: [
      { name: "telegram_token", set: true },
      { name: "calendar_password", set: false },
    ],
    lastRun: null,
    ...extra,
  };
}

function values(path: string, extra: Partial<SuggestionContext> = {}) {
  return suggestionsAt(context(path, extra)).map((suggestion) => suggestion.value);
}

describe("workflow suggestions", () => {
  it("autocomplete inside a loop offers loop values and earlier results, never later steps", () => {
    const offered = values("steps[2].body[0].text");
    expect(offered).toEqual(expect.arrayContaining(["loop.item", "loop.index", "steps.list.json", "steps.list.status", "inputs.service", "event.service.id"]));
    expect(offered.some((value) => value.startsWith("steps.after."))).toBe(false);
    expect(offered.some((value) => value.startsWith("steps.tell."))).toBe(false);
  });

  it("a variable declared once is offered everywhere after it, with the step that sets it", () => {
    const found = suggestionsAt(context("steps[2].body[1].branches[1][0].seconds")).find((suggestion) => suggestion.value === "vars.disk");
    expect(found?.description).toEqual({ key: "suggestions.variable", params: { name: "disk", step: "remember" } });
    expect(values("steps[0].value")).not.toContain("vars.disk");
  });

  it("a loop's own while sees loop values, its for_each does not", () => {
    expect(values("steps[2].while.left")).toContain("loop.item");
    expect(values("steps[2].for_each")).not.toContain("loop.item");
  });

  it("a called workflow's variables come as the step's vars, and an unset secret carries a warning", () => {
    expect(values("steps[4].text")).toContain("steps.call.vars.note");
    const secrets = suggestionsAt(context("steps[4].text")).filter((suggestion) => suggestion.group === "secrets");
    expect(secrets.map((secret) => [secret.value, secret.warning ?? null])).toEqual([
      ["secrets.telegram_token", null],
      ["secrets.calendar_password", "suggestions.secretUnset"],
    ]);
  });

  it("keys from the last run's answer are offered with their examples", () => {
    const lastRun = {
      entries: [
        { path: "steps[1]", step: "list", label: "list", kind: "http", iteration: null, outcome: "succeeded" as const, started_at: "", duration_milliseconds: 1, detail: "200", output: '{"state":"up","uptime":42,"disk":{"free":"12G"}}', shape: null },
      ],
      dropped: 0,
    };
    const found = suggestionsAt(context("steps[4].text", { lastRun })).filter((suggestion) => suggestion.value.startsWith("steps.list.json."));
    expect(found.map((suggestion) => [suggestion.value, suggestion.example])).toEqual([
      ["steps.list.json.state", "up"],
      ["steps.list.json.uptime", "42"],
      ["steps.list.json.disk", "{…}"],
      ["steps.list.json.disk.free", "12G"],
    ]);
  });
});

describe("json keys", () => {
  it("a body that is not JSON or cut short gives no keys, and a long value is shortened", () => {
    expect(jsonKeys("<html>")).toEqual([]);
    expect(jsonKeys('{"state":"up"')).toEqual([]);
    expect(jsonKeys(null)).toEqual([]);
    expect(jsonKeys(JSON.stringify({ text: "x".repeat(100) }))[0].example).toHaveLength(40);
  });
});

describe("template checks", () => {
  const scopeOf = (path: string, field = "") => scopeAt(steps, parsePath(path).path, ["service"], field);

  it("an unknown name in the text is found with its range and reason", () => {
    const text = "status {{steps.after.status}} ok";
    expect(checkTemplate(text, scopeOf("steps[1]"))).toEqual([{ start: 7, end: 29, name: "steps.after.status", reason: "unknownStep", params: { name: "after" } }]);
  });

  it("each reason mirrors the server's scope check", () => {
    const scope = scopeOf("steps[1]");
    expect(checkTemplate("{{inputs.other}}", scope)[0].reason).toBe("unknownInput");
    expect(checkTemplate("{{vars.disk}}", scope)).toEqual([]);
    expect(checkTemplate("{{vars.nope}}", scope)[0].reason).toBe("unknownVariable");
    expect(checkTemplate("{{loop.item}}", scope)[0].reason).toBe("outsideLoop");
    expect(checkTemplate("{{loop.item}}", scopeOf("steps[2].body[0]"))).toEqual([]);
    expect(checkTemplate("{{event.anything.goes}} {{secrets.any}}", scope)).toEqual([]);
    expect(checkTemplate("{{foo.bar}}", scope)[0].reason).toBe("notAValue");
    expect(checkTemplate("{{ not a name }}", scope)).toEqual([]);
  });

  it("the right side of a condition knows the values of a state, a status and a branch", () => {
    expect(knownValues("{{steps.probe.state}}", ["up", "down"]).map((entry) => entry.value)).toEqual(["up", "down"]);
    expect(knownValues("{{steps.ping.status}}", [])).toContainEqual({ value: "404", label: "404" });
    expect(knownValues("{{steps.check.branch}}", [])).toHaveLength(2);
    expect(knownValues("x {{steps.ping.status}}", [])).toEqual([]);
  });
});
