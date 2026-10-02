import { expect, it } from "vitest";

import { KEPT_STEPS, begin, isRedoKey, isUndoKey, nextRedo, nextUndo, pushed, redone, typingIn, undone } from "./history";

const same = (left: number, right: number) => left === right;

it("undoes and redoes each step, and a new step forgets what was undone", () => {
  let history = pushed(pushed(begin(12), 6, "half", same), 5, "five", same);
  expect(nextUndo(history)).toBe("five");
  history = undone(history);
  expect(history.present).toBe(6);
  expect(nextRedo(history)).toBe("five");
  expect(nextUndo(history)).toBe("half");
  history = undone(history);
  expect(history.present).toBe(12);
  expect(undone(history)).toBe(history);
  history = redone(history);
  expect(history.present).toBe(6);
  history = pushed(history, 8, "eight", same);
  expect(history.future).toEqual([]);
  expect(redone(history)).toBe(history);
});

it("a change to the same value is not a step, and only the last fifty steps are kept", () => {
  expect(pushed(begin(1), 1, "none", same).past).toEqual([]);
  let history = begin(0);
  for (let step = 1; step <= KEPT_STEPS + 10; step += 1) {
    history = pushed(history, step, `step ${step}`, same);
  }
  expect(history.past).toHaveLength(KEPT_STEPS);
  expect(history.past[0].value).toBe(10);
});

it("knows the undo and redo keys on every system, and leaves them to text fields", () => {
  expect(isUndoKey({ key: "z", ctrlKey: true, metaKey: false, shiftKey: false })).toBe(true);
  expect(isUndoKey({ key: "z", ctrlKey: false, metaKey: true, shiftKey: false })).toBe(true);
  expect(isUndoKey({ key: "Z", ctrlKey: true, metaKey: false, shiftKey: true })).toBe(false);
  expect(isRedoKey({ key: "Z", ctrlKey: true, metaKey: false, shiftKey: true })).toBe(true);
  expect(isRedoKey({ key: "y", ctrlKey: true, metaKey: false, shiftKey: false })).toBe(true);
  expect(typingIn(document.createElement("input"))).toBe(true);
  expect(typingIn(document.createElement("button"))).toBe(false);
});
