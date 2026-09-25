import { expect, it } from "vitest";

import { iconOf } from "./icons";

it("draws a bare name and a lucide prefix in the interface, asking the portal for nothing", () => {
  expect(iconOf({ id: "media", icon: "film" })).toEqual({ name: "film", source: null });
  expect(iconOf({ id: "media", icon: "lucide:film" })).toEqual({ name: "film", source: null });
});

it("asks the portal for every icon it fetches itself", () => {
  for (const icon of ["auto", "file:/srv/nas.png", "url:https://nas.example.com/icon.png", "catalog:jellyfin"]) {
    expect(iconOf({ id: "nas", icon })).toEqual({ name: null, source: "/api/icons/nas" });
  }
});

it("asks the public half when the page is public", () => {
  expect(iconOf({ id: "nas", icon: "auto" }, "public")).toEqual({ name: null, source: "/api/public/icons/nas" });
});

it("takes the address the public portal already resolved", () => {
  expect(iconOf({ id: "nas", icon: "/api/public/icons/nas" }, "public")).toEqual({
    name: null,
    source: "/api/public/icons/nas",
  });
});

it("draws the default icon for a service without one", () => {
  expect(iconOf({ id: "nas", icon: null })).toEqual({ name: null, source: null });
});
