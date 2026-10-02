import { expect, it } from "vitest";

import { type RawBlock, emptyPlace } from "./blocks";
import { blockAt, duplicateAt, fieldOfPath, insertAt, missingIn, moveTo, pathOfField, refusal, removeAt, slotAt, slotPaths, stepped, unmarked, withoutSlots } from "./block-tree";

const text = (value: string): RawBlock => ({ kind: "text", text: value });
const row = (...blocks: RawBlock[]): RawBlock => ({ kind: "row", blocks });
const column = (...blocks: RawBlock[]): RawBlock => ({ kind: "column", blocks });
const texts = (blocks: RawBlock[]): unknown => blocks.map((block) => (block.kind === "text" ? block.text : { [block.kind]: texts((block.blocks as RawBlock[]) ?? []) }));

const tree = (): RawBlock[] => [text("a"), row(text("b"), column(text("c"), text("d"))), text("e")];

it("finds, inserts, removes and duplicates by path", () => {
  expect(blockAt(tree(), [1, 1, 0])).toEqual(text("c"));
  expect(texts(insertAt(tree(), { parent: [1, 1], index: 1 }, text("x")))).toEqual(["a", { row: ["b", { column: ["c", "x", "d"] }] }, "e"]);
  expect(texts(removeAt(tree(), [1, 0]))).toEqual(["a", { row: [{ column: ["c", "d"] }] }, "e"]);
  const [copied, at] = duplicateAt(tree(), [1, 1, 0]);
  expect(at).toEqual([1, 1, 1]);
  expect(texts(copied)).toEqual(["a", { row: ["b", { column: ["c", "c", "d"] }] }, "e"]);
});

it("moves a block between groups and lands it where the marker was", () => {
  const [inside, landed] = moveTo(tree(), [0], { parent: [1, 1], index: 2 });
  expect(texts(inside)).toEqual([{ row: ["b", { column: ["c", "d", "a"] }] }, "e"]);
  expect(landed).toEqual([0, 1, 2]);
  expect(blockAt(inside, landed)).toEqual(text("a"));
  const [down, after] = moveTo(tree(), [0], { parent: [], index: 2 });
  expect(texts(down)).toEqual([{ row: ["b", { column: ["c", "d"] }] }, "a", "e"]);
  expect(after).toEqual([1]);
});

it("refuses a fifth block in a row, a group into itself and groups more than three deep", () => {
  const full = [row(text("1"), text("2"), text("3"), text("4"))];
  expect(refusal(full, text("x"), { parent: [0], index: 0 })).toBe("rowFull");
  expect(refusal(full, text("x"), { parent: [0], index: 0 }, [0, 3])).toBeNull();
  expect(refusal(tree(), blockAt(tree(), [1]) as RawBlock, { parent: [1, 1], index: 0 }, [1])).toBe("intoItself");
  const deep = [row(column(row(text("x"), text("y"))))];
  expect(refusal(deep, column(text("z")), { parent: [0, 0, 0], index: 0 })).toBe("tooDeep");
  expect(refusal(deep, text("z"), { parent: [0, 0, 0], index: 0 })).toBeNull();
});

it("keyboard steps move up, down, out of a group and into the group beside", () => {
  const up = stepped(tree(), [1, 1, 1], "up");
  expect(Array.isArray(up) && texts(up[0])).toEqual(["a", { row: ["b", { column: ["d", "c"] }] }, "e"]);
  expect(Array.isArray(up) && up[1]).toEqual([1, 1, 0]);
  const out = stepped(tree(), [1, 1, 0], "out");
  expect(Array.isArray(out) && texts(out[0])).toEqual(["a", { row: ["b", { column: ["d"] }, "c"] }, "e"]);
  const into = stepped(tree(), [1, 0], "into");
  expect(Array.isArray(into) && texts(into[0])).toEqual(["a", { row: [{ column: ["b", "c", "d"] }] }, "e"]);
  expect(stepped(tree(), [0], "up")).toBeNull();
});

it("names the empty fields a block needs and maps server fields to paths", () => {
  const blocks: RawBlock[] = [{ kind: "progress", value: "" }, row(text("ok"))];
  expect(missingIn(blocks)).toEqual([
    { path: [0], field: "value" },
    { path: [1], field: "blocks" },
  ]);
  expect(pathOfField("blocks[1].blocks[0].text")).toEqual([[1, 0], "text"]);
  expect(pathOfField("blocks[2]")).toEqual([[2], ""]);
  expect(fieldOfPath([1, 0])).toBe("blocks[1].blocks[0]");
});

it("an empty place in a group is no missing text, is left out when saved, and a group counts only real blocks", () => {
  const blocks: RawBlock[] = [row({ kind: "stat", label: "Free", value: "1" }, emptyPlace()), column(emptyPlace(), text(""))];
  expect(missingIn(blocks)).toEqual([
    { path: [0], field: "blocks" },
    { path: [1, 1], field: "text" },
  ]);
  expect(withoutSlots(blocks)).toEqual([row({ kind: "stat", label: "Free", value: "1" }), column(text(""))]);
  expect(slotPaths(blocks)).toEqual(["0.1", "1.0"]);
  expect(unmarked(blocks)[0].blocks).toEqual([{ kind: "stat", label: "Free", value: "1" }, text("")]);
  expect(slotAt(blocks, [0, 1])).toBe(true);
  expect(slotAt(blocks, [0, 0])).toBe(false);
});
