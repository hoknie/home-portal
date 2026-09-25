import { expect, it } from "vitest";

import { safeNext } from "./safe-next";

it("returns to a page of the portal and nowhere else", () => {
  expect(safeNext("/services/?id=nas")).toBe("/services/?id=nas");
  expect(safeNext(null)).toBe("/");
  expect(safeNext("https://evil.example")).toBe("/");
  expect(safeNext("//evil.example")).toBe("/");
  expect(safeNext("/login/")).toBe("/");
});
