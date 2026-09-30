import { describe, expect, it } from "vitest";

import type { InputDeclaration } from "@/entities/workflow";

import { entriesOf, entryOf, inputsRequest, kept, switched } from "./workflow-call";

const declarations: InputDeclaration[] = [
  { name: "service", type: "text", default: null, description: null },
  { name: "hosts", type: "list", default: ["nas"], description: "Hosts to check" },
  { name: "retries", type: "number", default: null, description: null },
  { name: "force", type: "boolean", default: null, description: null },
];

describe("a workflow call", () => {
  it("a list typed by hand is saved as a list", () => {
    const inputs = { hosts: { template: false, text: "", value: ["nas", "router"] } };
    expect(inputsRequest(inputs, declarations)).toEqual({ hosts: ["nas", "router"] });
  });

  it("a literal read back keeps its type and a template keeps its text", () => {
    const inputs = entriesOf({ retries: 3, force: true, service: "{{service.id}}", hosts: "{{webhook.body.hosts}}" });
    expect(inputs.retries).toEqual({ template: false, text: "", value: 3 });
    expect(inputsRequest(inputs, declarations)).toEqual({ retries: 3, force: true, service: "{{service.id}}", hosts: "{{webhook.body.hosts}}" });
  });

  it("a string that does not fit a typed input opens as a template, keeping its text", () => {
    expect(entryOf('["nas"]')).toEqual({ template: true, text: '["nas"]', value: null });
  });

  it("switching to a typed field reads the text when it fits, and back to a template writes the value", () => {
    const list = declarations[1];
    expect(switched({ template: true, text: '["a"]', value: null }, list)).toEqual({ template: false, text: '["a"]', value: ["a"] });
    expect(switched({ template: false, text: "", value: ["a", "b"] }, list)).toEqual({ template: true, text: '["a","b"]', value: ["a", "b"] });
  });

  it("empty inputs and inputs the workflow does not declare are left out", () => {
    const inputs = { hosts: { template: false, text: "", value: [] }, service: { template: true, text: "", value: null }, old: { template: true, text: "x", value: null } };
    expect(inputsRequest(inputs, declarations)).toEqual({});
    expect(Object.keys(kept(inputs, declarations))).toEqual(["hosts", "service"]);
  });
});
