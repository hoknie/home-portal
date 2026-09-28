import { describe, expect, it } from "vitest";

import { type Step, flowOf } from "@/entities/workflow";

import { dropTarget, slotPoints } from "./drop";
import { summaryOf } from "../summary";

const steps: Step[] = [
  { id: "ping", kind: "http", url: "http://nas.lan" },
  { id: "retry", kind: "loop", repeat: 3, body: [{ id: "pause", kind: "wait", seconds: 5 }] },
  { id: "tell", kind: "notify", text: "done" },
];

describe("drop targets", () => {
  const flow = flowOf(steps);
  const points = slotPoints(flow);
  const pointOf = (list: string, index: number) => points.find((point) => point.target.list === list && point.target.index === index)!;

  it("a node dropped near a slot moves there", () => {
    const body = pointOf("body", 1);
    expect(dropTarget(points, [{ list: "steps", index: 0 }], { x: body.x + 10, y: body.y - 10 })).toEqual({ owner: [{ list: "steps", index: 1 }], list: "body", index: 1 });
  });

  it("far from every slot, onto its own place or into itself, the node stays", () => {
    expect(dropTarget(points, [{ list: "steps", index: 0 }], { x: -500, y: -500 })).toBeNull();
    const own = pointOf("steps", 1);
    expect(dropTarget(points, [{ list: "steps", index: 0 }], own)).toBeNull();
    const inside = pointOf("body", 0);
    expect(dropTarget(points, [{ list: "steps", index: 1 }], inside)).toBeNull();
  });

  it("a node card summarises its settings", () => {
    expect(summaryOf(steps[0], [])).toEqual({ text: "GET http://nas.lan" });
    expect(summaryOf(steps[1], [])).toEqual({ key: "summaries.repeat", params: { count: 3 } });
    expect(summaryOf({ id: "call", kind: "workflow", workflow: "note" }, [{ id: "note", title: "Note" }])).toEqual({ text: "Note" });
  });
});
