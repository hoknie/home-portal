import { describe, expect, it } from "vitest";

import { type Step, flowOf, parsePath, workflowCatalogueSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";

import { destinationsFor } from "../edits/destinations";
import { dropTarget, slotKey, slotPoints } from "../edits/drop";
import { placeable, slotContext } from "./placing";
import { problemsOf } from "./validation";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

const loop = [{ list: "steps", index: 0 }];

const steps: Step[] = [
  { id: "each", kind: "loop", repeat: 2, body: [{ id: "fan", kind: "parallel", branches: [[{ id: "a", kind: "nothing" }], [{ id: "b", kind: "nothing" }]] }] },
  { id: "out", kind: "break" },
];

describe("where a step may go", () => {
  it("the root is neither a branch nor a loop, a loop body is both, and parallel cuts the loop off", () => {
    expect(slotContext({ owner: [], list: "steps", index: 0 })).toEqual({ inBranch: false, inLoop: false });
    expect(slotContext({ owner: loop, list: "body", index: 0 })).toEqual({ inBranch: true, inLoop: true });
    expect(slotContext({ owner: [...loop, { list: "body", index: 0 }], list: "then", index: 0 })).toEqual({ inBranch: true, inLoop: true });
    expect(slotContext({ owner: [...loop, { list: "body", index: 0 }], list: "branches[1]", index: 0 })).toEqual({ inBranch: true, inLoop: false });
  });

  it("a break, or an if holding one, goes only inside a loop", () => {
    const fork: Step = { id: "check", kind: "if", condition: { left: "a", op: "==", right: "b" }, then: [{ id: "x", kind: "continue" }] };
    expect(placeable(fork, { owner: [], list: "steps", index: 0 })).toBe(false);
    expect(placeable(fork, { owner: loop, list: "body", index: 0 })).toBe(true);
    expect(placeable({ id: "w", kind: "wait", seconds: 1 }, { owner: [], list: "steps", index: 0 })).toBe(true);
  });

  it("move to lists only the loop body for a break, and dragging skips every other slot", () => {
    const path = parsePath("steps[1]").path;
    expect(destinationsFor(steps, path).map((destination) => slotKey(destination.target))).toEqual(["steps[0]|body|1"]);
    const points = slotPoints(flowOf(steps));
    const root = points.find((point) => slotKey(point.target) === "|steps|0")!;
    const allowed = (target: Parameters<typeof placeable>[1]) => placeable(steps[1], target);
    expect(dropTarget(points, path, root, 48, allowed)).toBeNull();
  });

  it("the editor refuses a break outside a loop as the server does", () => {
    const draft = { id: "w", title: "W", enabled: true, description: null, tags: [], timeout_seconds: 300, inputs: [], steps };
    const problems = problemsOf(draft, catalogue, []);
    expect(problems["steps[1]"]).toBe("workflowEditor.problems.needsLoop");
    expect(problems["steps[0].body[0].branches[0][0]"]).toBeUndefined();
  });
});
