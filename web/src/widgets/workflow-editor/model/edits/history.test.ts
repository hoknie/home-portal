import { describe, expect, it } from "vitest";

import { COALESCE_MILLISECONDS, MOST_ENTRIES, historyOf, recorded, redone, undone } from "./history";

describe("editor history", () => {
  it("undo brings a deleted node back and redo deletes it again", () => {
    const before = { steps: ["a", "b"] };
    const after = { steps: ["a"] };
    const history = recorded(historyOf(before), after, null, 0);
    expect(undone(history).present).toBe(before);
    expect(redone(undone(history)).present).toBe(after);
    expect(undone(historyOf(before))).toEqual(historyOf(before));
  });

  it("typing into one field coalesces within the window, and a new change clears redo", () => {
    let history = historyOf("");
    history = recorded(history, "p", "steps[0].url", 1000);
    history = recorded(history, "pi", "steps[0].url", 1000 + COALESCE_MILLISECONDS - 1);
    history = recorded(history, "pin", "steps[0].url", 1000 + COALESCE_MILLISECONDS - 1 + COALESCE_MILLISECONDS);
    expect(history.past).toEqual(["", "pi"]);
    const back = undone(history);
    expect(recorded(back, "other", null, 5000).future).toEqual([]);
  });

  it("keeps at most the limit of entries", () => {
    let history = historyOf(0);
    for (let value = 1; value <= MOST_ENTRIES + 20; value += 1) {
      history = recorded(history, value, null, value);
    }
    expect(history.past).toHaveLength(MOST_ENTRIES);
    expect(history.past[0]).toBe(20);
  });
});
