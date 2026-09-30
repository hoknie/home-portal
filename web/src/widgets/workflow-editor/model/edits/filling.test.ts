import { expect, it } from "vitest";

import type { Step } from "@/entities/workflow";

import { filledSteps } from "./filling";

const ping: Step = { id: "ping", kind: "http", url: "http://nas.lan" };

it("an if with only then gets a step that does nothing in else", () => {
  const [check] = filledSteps([{ id: "check", kind: "if", condition: { all: [] }, then: [ping] }]);
  expect(check.then).toEqual([ping]);
  expect(check.else).toEqual([{ id: "nothing", kind: "nothing" }]);
});

it("empty blocks at any depth are filled, each with an id no other step has", () => {
  const steps: Step[] = [
    { id: "nothing", kind: "log", message: "taken" },
    { id: "both", kind: "parallel", branches: [[{ id: "again", kind: "loop", repeat: 3, body: [] }], []] },
  ];
  const [, both] = filledSteps(steps);
  const loop = both.branches?.[0][0] as Step;
  expect(loop.body).toEqual([{ id: "nothing_2", kind: "nothing" }]);
  expect(both.branches?.[1]).toEqual([{ id: "nothing_3", kind: "nothing" }]);
});

it("a tree without empty blocks comes back equal", () => {
  const steps: Step[] = [ping, { id: "check", kind: "if", condition: { all: [] }, then: [{ id: "a", kind: "stop" }], else: [{ id: "b", kind: "stop" }] }];
  expect(filledSteps(steps)).toEqual(steps);
});

it("an empty workflow stays empty", () => {
  expect(filledSteps([])).toEqual([]);
});
