import { describe, expect, it } from "vitest";

import type { Step } from "@/entities/workflow";

import { summaryOf } from "./summary";
import { MARK_CLOSE, MARK_OPEN, partsOf, withValues } from "./values-on-nodes";

describe("values on nodes", () => {
  it("a whole field that equals a rendered template shows its value, decoded from JSON, as a chip", () => {
    const step: Step = { id: "ping", kind: "http", url: "http://{{inputs.host}}/ping" };
    const { step: shown, shown: values } = withValues(step, [{ template: "http://{{inputs.host}}/ping", value: '"http://nas.lan/ping"' }]);
    const summary = summaryOf(shown, []);
    const text = "text" in summary ? summary.text : "";
    expect(partsOf(text, values)).toEqual([{ text: "GET " }, { text: "http://nas.lan/ping", template: "http://{{inputs.host}}/ping" }]);
  });

  it("only whole fields match, numbers and objects read as JSON, and a field without a value stays a template", () => {
    const step: Step = { id: "tell", kind: "notify", text: "{{inputs.host}} is down", title: "{{vars.title}}" };
    const { step: shown } = withValues(step, [
      { template: "{{inputs.host}}", value: '"nas"' },
      { template: "{{vars.title}}", value: '{"a":1}' },
    ]);
    expect((shown as { text: string }).text).toBe("{{inputs.host}} is down");
    expect((shown as { title: string }).title).toContain('{"a":1}');
  });

  it("list and table entries are matched one by one, and child steps are left alone", () => {
    const step: Step = { id: "run", kind: "script", script: "restart.sh", args: ["--id", "{{inputs.host}}"], env: { HOST: "{{inputs.host}}" } };
    const { step: shown, shown: values } = withValues(step, [{ template: "{{inputs.host}}", value: '"nas"' }]);
    expect((shown as { args: string[] }).args[1]).toBe(`${MARK_OPEN}0nas${MARK_CLOSE}`);
    expect(values).toHaveLength(2);
    const branchy: Step = { id: "check", kind: "if", condition: { left: "{{inputs.host}}", op: "==", right: "nas" }, then: [{ id: "inner", kind: "log", message: "{{inputs.host}}" }] };
    const { step: kept } = withValues(branchy, [{ template: "{{inputs.host}}", value: '"nas"' }]);
    expect((kept as { then: { message: string }[] }).then[0].message).toBe("{{inputs.host}}");
  });

  it("text without markers is one plain part", () => {
    expect(partsOf("wait 20 s", [])).toEqual([{ text: "wait 20 s" }]);
  });
});
