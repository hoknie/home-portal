import { expect, it } from "vitest";

import { BLOCK_KINDS, blocksOf, errorsUnder, newBlock, tabOfError, toneModeOf, withToneMode } from "./blocks";

it("every kind of block starts with the fields it needs", () => {
  for (const kind of BLOCK_KINDS) {
    expect(newBlock(kind).kind).toBe(kind);
  }
  expect(newBlock("row").blocks).toHaveLength(2);
});

it("reads only the blocks of the settings, and a new group starts with two empty slots", () => {
  expect(blocksOf({ blocks: [{ kind: "text" }, null, 4] })).toEqual([{ kind: "text" }]);
  expect(newBlock("column").blocks).toEqual([
    { kind: "text", text: "", slot: true },
    { kind: "text", text: "", slot: true },
  ]);
});

it("switches the way a block gets its tone, keeping only one rule", () => {
  const block = { kind: "badge", text: "x", tone: "ok" };
  expect(toneModeOf(block)).toBe("fixed");
  const thresholds = withToneMode(block, "thresholds");
  expect(thresholds).toEqual({ kind: "badge", text: "x", thresholds: { warning: 75, danger: 90 } });
  expect(toneModeOf(withToneMode(thresholds, "tones"))).toBe("tones");
});

it("places each error on its tab and under its block", () => {
  expect(tabOfError("source.workflow")).toBe("data");
  expect(tabOfError("blocks[1].text")).toBe("content");
  expect(errorsUnder([{ field: "blocks[1].text", message: "bad" }, { field: "blocks[2]", message: "no" }], "blocks[1]")).toEqual({ text: "bad" });
});
