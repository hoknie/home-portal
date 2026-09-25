import { expect, it } from "vitest";

import { snapSize, stepSize } from "./resize";

it("snaps a dragged edge to the nearest of the five widths", () => {
  expect(snapSize("full", -600, 1200)).toBe("half");
  expect(snapSize("full", -560, 1200)).toBe("half");
  expect(snapSize("half", 180, 1200)).toBe("two-thirds");
  expect(snapSize("half", 20, 1200)).toBe("half");
  expect(snapSize("quarter", 60, 1200)).toBe("third");
});

it("stops at the narrowest and the widest width", () => {
  expect(snapSize("quarter", -900, 1200)).toBe("quarter");
  expect(snapSize("full", 900, 1200)).toBe("full");
  expect(snapSize("half", 100, 0)).toBe("half");
});

it("steps one width at a time and stays within the ends", () => {
  expect(stepSize("half", 1)).toBe("two-thirds");
  expect(stepSize("half", -1)).toBe("third");
  expect(stepSize("full", 1)).toBe("full");
  expect(stepSize("quarter", -1)).toBe("quarter");
});
