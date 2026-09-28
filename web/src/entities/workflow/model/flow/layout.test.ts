import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { type Step, workflowsSchema } from "../schema";
import { flowOf } from "./build";
import { flowOrder, neighbour } from "./order";
import type { Box } from "./sizes";

const samples = workflowsSchema.parse(apiSamples.workflows).workflows;

const deep: Step[] = [
  {
    id: "outer",
    kind: "loop",
    repeat: 2,
    body: [
      {
        id: "check",
        kind: "if",
        condition: { left: "a", op: "==", right: "b" },
        then: [{ id: "all", kind: "parallel", branches: [[{ id: "x", kind: "wait", seconds: 1 }], [{ id: "y", kind: "wait", seconds: 1 }, { id: "z", kind: "wait", seconds: 1 }]] }],
        else: [{ id: "w", kind: "wait", seconds: 1 }],
      },
    ],
  },
  { id: "last", kind: "wait", seconds: 1 },
];

function overlaps(left: Box, right: Box) {
  return left.x < right.x + right.width && right.x < left.x + left.width && left.y < right.y + right.height && right.y < left.y + left.height;
}

function inside(outer: Box, inner: Box) {
  return inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.width <= outer.x + outer.width && inner.y + inner.height <= outer.y + outer.height;
}

describe("workflow layout", () => {
  it("the same tree always gets the same positions", () => {
    expect(flowOf(deep).nodes.map((node) => node.box)).toEqual(flowOf(structuredClone(deep)).nodes.map((node) => node.box));
  });

  it.each([["the samples", samples.flatMap((workflow) => [workflow.steps])], ["a three-deep nesting", [deep]]])("no two nodes overlap in %s", (_, trees) => {
    for (const steps of trees) {
      const nodes = flowOf(steps).nodes.filter((node) => node.type !== "frame");
      for (const [index, node] of nodes.entries()) {
        for (const other of nodes.slice(index + 1)) {
          expect(overlaps(node.box, other.box), `${node.id} and ${other.id}`).toBe(false);
        }
      }
    }
  });

  it("a frame contains its loop's body and its join", () => {
    const flow = flowOf(deep);
    const frame = flow.nodes.find((node) => node.id === "steps[0]:frame")!;
    for (const node of flow.nodes.filter((entry) => entry.id.startsWith("steps[0].body") || entry.id === "steps[0]:join")) {
      expect(inside(frame.box, node.box), node.id).toBe(true);
    }
    expect(inside(frame.box, flow.nodes.find((node) => node.id === "steps[0]")!.box)).toBe(false);
  });

  it("keyboard order follows the run: next, previous, into a branch and out", () => {
    expect(flowOrder(deep).map((path) => path.map((place) => `${place.list}[${place.index}]`).join("."))).toEqual([
      "steps[0]",
      "steps[0].body[0]",
      "steps[0].body[0].then[0]",
      "steps[0].body[0].then[0].branches[0][0]",
      "steps[0].body[0].then[0].branches[1][0]",
      "steps[0].body[0].then[0].branches[1][1]",
      "steps[0].body[0].else[0]",
      "steps[1]",
    ]);
    const first = [{ list: "steps", index: 0 }];
    expect(neighbour(deep, null, "next")).toEqual(first);
    expect(neighbour(deep, first, "next")).toEqual([...first, { list: "body", index: 0 }]);
    expect(neighbour(deep, first, "into")).toEqual([...first, { list: "body", index: 0 }]);
    expect(neighbour(deep, [...first, { list: "body", index: 0 }], "out")).toEqual(first);
    expect(neighbour(deep, first, "previous")).toEqual(first);
  });
});
