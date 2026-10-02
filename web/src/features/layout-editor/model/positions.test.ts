import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import { fromLayout, updateWidget } from "./draft";
import { clampedCell, movedTo, overlaps, settled } from "./positions";

const cell = (column: number, row: number, width: number, rows = 1) => ({ column, row, width, rows });

it("knows when two cells share a place", () => {
  expect(overlaps(cell(1, 1, 4), cell(4, 1, 4))).toBe(true);
  expect(overlaps(cell(1, 1, 4), cell(5, 1, 4))).toBe(false);
  expect(overlaps(cell(1, 1, 4, 2), cell(2, 2, 2))).toBe(true);
});

it("keeps a cell inside the twelve columns and below the first row", () => {
  expect(clampedCell(cell(11, 0, 4))).toEqual(cell(9, 1, 4));
});

it("the moved widget keeps its place and every widget it lands on goes below it, again and again", () => {
  const result = settled({ a: cell(1, 1, 6, 2), b: cell(1, 3, 6), moved: cell(1, 1, 12, 3) }, "moved");
  expect(result.moved).toEqual(cell(1, 1, 12, 3));
  expect(result.a).toEqual(cell(1, 4, 6, 2));
  expect(result.b).toEqual(cell(1, 6, 6));
});

it("moving a narrow widget below on the other side pins its neighbours and leaves a gap", () => {
  const loaded = updateWidget(fromLayout(dashboardSchema.parse(apiSamples.dashboard)), "#0", { width: 4 });
  const measured = { "#0": cell(1, 1, 8, 2), "#1": cell(9, 1, 4, 3) };
  const moved = movedTo(loaded, "#0", "now", { column: 9, row: 4 }, { ...measured, "#0": cell(1, 1, 4, 2) });
  const summary = moved.widgets.find((widget) => widget.uid === "#0");
  expect(summary).toMatchObject({ column: 9, row: 4, section: "now" });
  expect(moved.widgets.find((widget) => widget.uid === "#1")).toMatchObject({ column: 9, row: 1 });
});
