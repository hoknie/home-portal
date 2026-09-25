import { expect, it } from "vitest";

import { NAVIGATION, isActive } from "./navigation";

it("marks a management section on its own page and its subpages only", () => {
  expect(isActive("/", "/")).toBe(true);
  expect(isActive("/admin/services/", "/")).toBe(false);
  expect(isActive("/admin/services", "/admin/services/")).toBe(true);
  expect(isActive("/admin/network/", "/admin/layout/")).toBe(false);
});

it("offers the four management pages", () => {
  expect(NAVIGATION.map((item) => item.href)).toEqual(["/admin/services/", "/admin/layout/", "/admin/network/", "/admin/proxy/"]);
});
