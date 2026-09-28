import { describe, expect, it } from "vitest";

import type { Step } from "../schema";
import { flowOf } from "./build";

const retry: Step[] = [
  {
    id: "retry",
    kind: "loop",
    repeat: 3,
    body: [
      { id: "ping", kind: "http", method: "GET", url: "http://nas.lan/api/ping" },
      { id: "check", kind: "if", condition: { left: "{{steps.ping.status}}", op: "==", right: "200" }, then: [{ id: "done", kind: "stop", outcome: "succeeded" }] },
    ],
  },
  { id: "tell", kind: "notify", text: "still down" },
];

function links(steps: Step[]) {
  return flowOf(steps).edges.map((edge) => `${edge.source} -> ${edge.target}${edge.label ? ` [${edge.label.key}]` : ""}`);
}

describe("workflow flow", () => {
  it("an empty workflow is start, one empty slot and end", () => {
    const flow = flowOf([]);
    expect(flow.nodes.map((node) => node.type)).toEqual(["start", "empty", "end"]);
    expect(flow.slots).toEqual([{ owner: [], list: "steps", index: 0 }]);
  });

  it("the shape of a condition: then through two steps, else through its own slot, both joining before the next step", () => {
    const steps: Step[] = [
      { id: "check", kind: "if", condition: { left: "a", op: "==", right: "1" }, then: [{ id: "a", kind: "wait", seconds: 1 }, { id: "b", kind: "wait", seconds: 1 }], else: [] },
      { id: "tell", kind: "notify", text: "x" },
    ];
    expect(links(steps)).toEqual([
      "start -> steps[0]",
      "steps[0] -> steps[0].then[0] [then]",
      "steps[0].then[0] -> steps[0].then[1]",
      "steps[0].then[1] -> steps[0]:join",
      "steps[0] -> steps[0]:else:empty [else]",
      "steps[0]:else:empty -> steps[0]:join",
      "steps[0]:join -> steps[1]",
      "steps[1] -> end",
    ]);
    const flow = flowOf(steps);
    expect(flow.nodes.find((node) => node.id === "steps[0]:else:empty")?.target).toEqual({ owner: [{ list: "steps", index: 0 }], list: "else", index: 0 });
    expect(flow.edges.find((edge) => edge.id === "steps[0]:join->steps[1]")?.slot).toEqual({ owner: [], list: "steps", index: 1 });
  });

  it("a loop gets a frame, a mode label and a return arrow", () => {
    const flow = flowOf(retry);
    expect(flow.nodes.filter((node) => node.type === "frame").map((node) => node.id)).toEqual(["steps[0]:frame"]);
    expect(flow.edges.find((edge) => edge.target === "steps[0].body[0]")?.label).toEqual({ key: "repeat", params: { count: 3 } });
    expect(flow.edges.find((edge) => edge.kind === "loop-back")).toMatchObject({ source: "steps[0]:join", target: "steps[0]" });
  });

  it("parallel fans out into its branches and joins", () => {
    const steps: Step[] = [
      {
        id: "all",
        kind: "parallel",
        branches: [[{ id: "a", kind: "status", service: "a" }], [{ id: "b", kind: "status", service: "b" }], []],
      },
    ];
    expect(links(steps)).toEqual([
      "start -> steps[0]",
      "steps[0] -> steps[0].branches[0][0] [branch]",
      "steps[0].branches[0][0] -> steps[0]:join",
      "steps[0] -> steps[0].branches[1][0] [branch]",
      "steps[0].branches[1][0] -> steps[0]:join",
      "steps[0] -> steps[0]:branches[2]:empty [branch]",
      "steps[0]:branches[2]:empty -> steps[0]:join",
      "steps[0]:join -> end",
    ]);
  });

  it("every insertion point of the retry tree is a slot", () => {
    const slots = flowOf(retry).slots.map((slot) => `${slot.owner.map((place) => `${place.list}[${place.index}]`).join(".")}|${slot.list}|${slot.index}`);
    expect(slots).toEqual(
      expect.arrayContaining(["|steps|0", "|steps|1", "|steps|2", "steps[0]|body|0", "steps[0]|body|1", "steps[0]|body|2", "steps[0].body[1]|then|0", "steps[0].body[1]|then|1", "steps[0].body[1]|else|0"]),
    );
  });
});
