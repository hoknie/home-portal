import { expect, it } from "vitest";

import { itemPaths, knownPaths, suggestionsOf } from "./suggestion-source";

const groups = { data: "Data", widget: "Widget", item: "Item" };

it("suggests declared outputs with their descriptions before anything has run", () => {
  const known = knownPaths(
    [
      { path: "data.temperature", description: "°C now", kind: "value" },
      { path: "data.wind", description: null, kind: "value" },
    ],
    null,
  );
  const suggestions = suggestionsOf(known, [], groups, (kind) => kind);
  expect(suggestions.slice(0, 2)).toEqual([
    { value: "data.temperature", group: "Data", description: "°C now · value", example: undefined },
    { value: "data.wind", group: "Data", description: "value", example: undefined },
  ]);
  expect(suggestions.map((suggestion) => suggestion.value)).toContain("fetched_at");
});

it("a run adds the paths it found, with a sample and a type, to the declared ones", () => {
  const known = knownPaths([{ path: "data.temperature", description: "°C now", kind: "value" }], { temperature: 20, place: "Riga" });
  expect(known).toEqual([
    { path: "data.temperature", description: "°C now", kind: "number", sample: "20" },
    { path: "data.place", description: null, kind: "text", sample: "Riga" },
  ]);
});

it("inside a list, item paths come from the first element of the list its items name", () => {
  const data = { clients: [{ name: "phone", ip: "10.0.0.2" }] };
  const items = itemPaths("{{data.clients}}", data, knownPaths([], data)).map((path) => path.path);
  expect(items).toEqual(["item", "item.name", "item.ip", "index"]);
  const declared = knownPaths([{ path: "data.clients", description: null, kind: "list" }, { path: "data.clients.0.name", description: null, kind: "text" }], null);
  expect(itemPaths("{{data.clients}}", null, declared).map((path) => path.path)).toEqual(["item.name", "index"]);
});
