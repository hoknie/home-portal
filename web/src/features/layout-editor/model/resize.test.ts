import { expect, it } from "vitest";

import { snapColumns, snapRows, snapSize, stepColumns, stepRows, steppedSize } from "./resize";

it("snaps a dragged edge to the nearest whole column", () => {
  expect(snapColumns(12, -600, 1200)).toBe(6);
  expect(snapColumns(12, -700, 1200)).toBe(5);
  expect(snapColumns(6, 180, 1200)).toBe(8);
  expect(snapColumns(6, 20, 1200)).toBe(6);
});

it("stops at one column and at the whole row", () => {
  expect(snapColumns(3, -900, 1200)).toBe(1);
  expect(snapColumns(12, 900, 1200)).toBe(12);
  expect(snapColumns(6, 100, 0)).toBe(6);
});

it("steps one column at a time and stays within the ends", () => {
  expect(stepColumns(6, 1)).toBe(7);
  expect(stepColumns(6, -1)).toBe(5);
  expect(stepColumns(12, 1)).toBe(12);
  expect(stepColumns(1, -1)).toBe(1);
});

it("snaps a dragged bottom edge to whole 80 px rows, and above one row to the content's height", () => {
  expect(snapRows(80, 192)).toBe(3);
  expect(snapRows(80, 1000)).toBe(8);
  expect(snapRows(176, -150)).toBe("auto");
  expect(snapRows(176, -70)).toBe(1);
});

it("steps rows from the content's height to one row and back", () => {
  expect(stepRows("auto", 1)).toBe(1);
  expect(stepRows(1, -1)).toBe("auto");
  expect(stepRows("auto", -1)).toBe("auto");
  expect(stepRows(8, 1)).toBe(8);
});

it("a corner changes both, an edge only its own", () => {
  const start = { width: 12, height: "auto" as const };
  expect(snapSize(start, "both", { x: -600, y: 220 }, { gridWidth: 1200, startPixels: 60 })).toEqual({ width: 6, height: 3 });
  expect(snapSize(start, "width", { x: -600, y: 220 }, { gridWidth: 1200, startPixels: 60 })).toEqual({ width: 6, height: "auto" });
  expect(snapSize(start, "height", { x: -600, y: 220 }, { gridWidth: 1200, startPixels: 60 })).toEqual({ width: 12, height: 3 });
});

it("the arrows step the width on its edge, the height on the bottom, and both on the corner", () => {
  const size = { width: 6, height: 2 as const };
  expect(steppedSize(size, "width", "ArrowRight")).toEqual({ width: 7, height: 2 });
  expect(steppedSize(size, "height", "ArrowDown")).toEqual({ width: 6, height: 3 });
  expect(steppedSize(size, "height", "ArrowUp")).toEqual({ width: 6, height: 1 });
  expect(steppedSize(size, "both", "ArrowLeft")).toEqual({ width: 5, height: 2 });
  expect(steppedSize(size, "both", "ArrowDown")).toEqual({ width: 6, height: 3 });
  expect(steppedSize(size, "both", "Tab")).toBeNull();
});
