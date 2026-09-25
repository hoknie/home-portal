import { expect, it } from "vitest";

import { destinationOf } from "./destination";

it("a path to return to stays inside the interface", () => {
  expect(destinationOf(null, "/admin/services/")).toEqual({ href: "/admin/services/", leavesTheInterface: false });
  expect(destinationOf("/admin/layout/", null)).toEqual({ href: "/admin/layout/", leavesTheInterface: false });
});

it("an absolute address is handed to the portal to check, never followed directly", () => {
  expect(destinationOf(null, "https://nas.example.com/photos")).toEqual({
    href: "/api/proxy/continue?to=https%3A%2F%2Fnas.example.com%2Fphotos",
    leavesTheInterface: true,
  });
  expect(destinationOf(null, "https://evil.example.net/").href.startsWith("/api/proxy/continue?to=")).toBe(true);
});

it("anything else returns home", () => {
  expect(destinationOf(null, "//evil.example.net")).toEqual({ href: "/", leavesTheInterface: false });
  expect(destinationOf(null, "javascript:alert(1)")).toEqual({ href: "/", leavesTheInterface: false });
  expect(destinationOf(null, null)).toEqual({ href: "/", leavesTheInterface: false });
});
