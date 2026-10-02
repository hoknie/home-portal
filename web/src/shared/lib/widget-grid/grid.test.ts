import { expect, it } from "vitest";

import { GAP_PIXELS, PLACED_CLASSES, STEP_PIXELS, STEPS_PER_ROW, cellClasses, cellStyle, inReadingOrder, positionOf, rowsFor, tabletSpanOf } from "./grid";

it("a widget spans its columns on a wide screen, half or the whole row on a tablet, and its rows", () => {
  expect(cellStyle(5, 3, 1)).toEqual({ "--span": 5, "--tablet-span": 6, "--rows": 36 });
  expect(cellStyle(7, "auto", 2)).toEqual({ "--span": 7, "--tablet-span": 12, "--rows": 2 });
  expect(cellStyle(13, "auto", 1)).toMatchObject({ "--span": 12 });
  expect([1, 6, 7, 12].map(tabletSpanOf)).toEqual([6, 6, 12, 12]);
});

it("a fixed height keeps whole 80 px rows with their gaps, twelve 8 px steps each", () => {
  expect(STEPS_PER_ROW).toBe(12);
  expect(cellStyle(4, 2, 0)).toMatchObject({ "--rows": 24 });
  expect(24 * STEP_PIXELS - GAP_PIXELS).toBe(2 * 80 + GAP_PIXELS);
});

it("an automatic height follows its content to the next 8 px, so no row is left half empty", () => {
  expect(rowsFor(0)).toBe(2);
  expect(rowsFor(80)).toBe(12);
  expect(rowsFor(81)).toBe(13);
  expect(rowsFor(100)).toBe(15);
  expect(rowsFor(100) * STEP_PIXELS - GAP_PIXELS - 100).toBeLessThan(STEP_PIXELS);
});

it("a widget with a column keeps it from 1024 px, and its row only sets its order, so empty rows close up and nothing overlaps", () => {
  expect(positionOf({ column: 9, row: 2 })).toEqual({ column: 9, row: 2 });
  expect(positionOf({ column: 9, row: null })).toBeNull();
  expect(cellStyle(4, "auto", 2, { column: 9, row: 2 })).toEqual({ "--span": 4, "--tablet-span": 6, "--rows": 2, "--column": 9, "--row": 2 });
  expect(PLACED_CLASSES).not.toContain("grid-row:var(--row)");
  expect(cellClasses({ column: 9, row: 2 })).toBe(PLACED_CLASSES);
  expect(PLACED_CLASSES).toContain("lg:[grid-column:var(--column)/span_var(--span)]");
  expect(PLACED_CLASSES).toContain("col-span-12");
});

it("a narrow widget below on the other side comes after the first row in reading order", () => {
  const below = { key: "below", column: 9, row: 2 };
  const first = { key: "first", column: 1, row: 1 };
  const flowing = { key: "flowing", column: null, row: null };
  expect(inReadingOrder([below, flowing, first]).map((widget) => widget.key)).toEqual(["first", "below", "flowing"]);
});
