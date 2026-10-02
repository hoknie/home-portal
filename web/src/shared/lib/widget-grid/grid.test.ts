import { expect, it } from "vitest";

import { PLACED_CLASSES, cellClasses, cellStyle, inReadingOrder, positionOf, rowsFor, tabletSpanOf } from "./grid";

it("a widget spans its columns on a wide screen, half or the whole row on a tablet, and its rows", () => {
  expect(cellStyle(5, 3, 1)).toEqual({ "--span": 5, "--tablet-span": 6, "--rows": 3 });
  expect(cellStyle(7, "auto", 2)).toEqual({ "--span": 7, "--tablet-span": 12, "--rows": 2 });
  expect(cellStyle(13, "auto", 1)).toMatchObject({ "--span": 12 });
  expect([1, 6, 7, 12].map(tabletSpanOf)).toEqual([6, 6, 12, 12]);
});

it("an automatic height takes as many 80 px rows as its content needs, gaps included", () => {
  expect(rowsFor(0)).toBe(1);
  expect(rowsFor(80)).toBe(1);
  expect(rowsFor(81)).toBe(2);
  expect(rowsFor(176)).toBe(2);
  expect(rowsFor(177)).toBe(3);
});

it("a widget with a column and a row is placed there from 1024 px, and flows like the others below", () => {
  expect(positionOf({ column: 9, row: 2 })).toEqual({ column: 9, row: 2 });
  expect(positionOf({ column: 9, row: null })).toBeNull();
  expect(cellStyle(4, "auto", 2, { column: 9, row: 2 })).toEqual({ "--span": 4, "--tablet-span": 6, "--rows": 2, "--column": 9, "--row": 2 });
  expect(cellClasses({ column: 9, row: 2 })).toBe(PLACED_CLASSES);
  expect(PLACED_CLASSES).toContain("lg:[grid-column:var(--column)/span_var(--span)]");
  expect(PLACED_CLASSES).toContain("lg:[grid-row:var(--row)/span_var(--rows)]");
  expect(PLACED_CLASSES).toContain("col-span-12");
});

it("a narrow widget below on the other side comes after the first row in reading order", () => {
  const below = { key: "below", column: 9, row: 2 };
  const first = { key: "first", column: 1, row: 1 };
  const flowing = { key: "flowing", column: null, row: null };
  expect(inReadingOrder([below, flowing, first]).map((widget) => widget.key)).toEqual(["first", "below", "flowing"]);
});
