import { expect, it } from "vitest";

import { byGroup } from "./groups";

it("groups come sorted by name in the page's language, services keep their order, and those without a group come last", () => {
  const services = [
    { id: "jellyfin", group: "Media" },
    { id: "router", group: "Network" },
    { id: "nas", group: null },
    { id: "immich", group: "Media" },
    { id: "printer", group: " " },
  ];
  expect(byGroup(services, "en").map((group) => [group.name, group.items.map((service) => service.id)])).toEqual([
    ["Media", ["jellyfin", "immich"]],
    ["Network", ["router"]],
    [null, ["nas", "printer"]],
  ]);
  expect(byGroup([{ group: "Ñandú" }, { group: "Nube" }], "es").map((group) => group.name)).toEqual(["Nube", "Ñandú"]);
});
