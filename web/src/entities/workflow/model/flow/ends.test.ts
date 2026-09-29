import { describe, expect, it } from "vitest";

import type { Step } from "../schema";
import { closes, closingOf, insideLoop, needsLoop, unreachableSteps } from "./ends";

const fork = (then: Step[], otherwise: Step[]): Step => ({ id: "check", kind: "if", condition: { left: "a", op: "==", right: "b" }, then, else: otherwise });

describe("workflow ends", () => {
  it("stop, break and continue close a list, each in its own way", () => {
    expect(closingOf({ id: "a", kind: "stop", outcome: "failed" })).toBe("failed");
    expect(closingOf({ id: "a", kind: "stop", outcome: "succeeded" })).toBe("succeeded");
    expect(closingOf({ id: "a", kind: "break" })).toBe("break");
    expect(closingOf({ id: "a", kind: "continue" })).toBe("continue");
    expect(closingOf({ id: "a", kind: "nothing" })).toBeNull();
  });

  it("an if closes only when every branch closes, and an empty else keeps it open", () => {
    expect(closes(fork([{ id: "x", kind: "break" }], [{ id: "y", kind: "stop", outcome: "failed" }]))).toBe(true);
    expect(closes(fork([{ id: "x", kind: "break" }], []))).toBe(false);
  });

  it("steps after an ending are unreachable, in every nested list", () => {
    const steps: Step[] = [
      {
        id: "each",
        kind: "loop",
        repeat: 1,
        body: [
          { id: "out", kind: "break" },
          { id: "never", kind: "nothing" },
        ],
      },
      fork([{ id: "p", kind: "stop", outcome: "succeeded" }], [{ id: "q", kind: "stop", outcome: "failed" }]),
      { id: "late", kind: "nothing" },
    ];
    expect(unreachableSteps(steps)).toEqual([
      [
        { list: "steps", index: 0 },
        { list: "body", index: 1 },
      ],
      [{ list: "steps", index: 2 }],
    ]);
  });

  it("a slot is inside a loop through if steps, never through parallel or at the root", () => {
    const loop = [{ list: "steps", index: 0 }];
    expect(insideLoop({ owner: loop, list: "body", index: 0 })).toBe(true);
    expect(insideLoop({ owner: [...loop, { list: "body", index: 0 }], list: "then", index: 0 })).toBe(true);
    expect(insideLoop({ owner: [...loop, { list: "body", index: 0 }], list: "branches[0]", index: 0 })).toBe(false);
    expect(insideLoop({ owner: [], list: "steps", index: 0 })).toBe(false);
    expect(insideLoop({ owner: loop, list: "else", index: 0 })).toBe(false);
  });

  it("an if that holds a break needs a loop, but a loop that holds one does not", () => {
    expect(needsLoop(fork([{ id: "x", kind: "break" }], []))).toBe(true);
    expect(needsLoop({ id: "l", kind: "loop", repeat: 1, body: [{ id: "x", kind: "break" }] })).toBe(false);
    expect(needsLoop({ id: "c", kind: "continue" })).toBe(true);
  });
});
