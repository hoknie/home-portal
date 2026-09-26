import { expect, it, vi } from "vitest";

import { cameBackFrom, destinationOf, hostOf, leaveTo } from "./destination";

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

it("a return to the same place soon after leaving for it is noticed", () => {
  const assign = vi.fn();
  vi.stubGlobal("location", { assign });
  leaveTo("/api/proxy/continue?to=x");
  expect(assign).toHaveBeenCalledWith("/api/proxy/continue?to=x");
  expect(cameBackFrom("/api/proxy/continue?to=x")).toBe(true);
  expect(cameBackFrom("/api/proxy/continue?to=y")).toBe(false);
  expect(cameBackFrom("/api/proxy/continue?to=x", Date.now() + 60_000)).toBe(false);
  vi.unstubAllGlobals();
});

it("names the host of an address, and nothing for anything else", () => {
  expect(hostOf("https://torrent.portal.home/")).toBe("torrent.portal.home");
  expect(hostOf("/admin/")).toBe("");
  expect(hostOf(null)).toBe("");
});
