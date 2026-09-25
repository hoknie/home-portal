import { expect, it } from "vitest";

import { type GridWidget, placeWidgets } from "./layout";

const widget = (key: string, section: string | null): GridWidget => ({
  key,
  type: "status-summary",
  id: null,
  title: null,
  settings: {},
  section,
  size: "full",
});

const sections = [
  { id: "now", title: "Now" },
  { id: "media", title: "Media" },
  { id: "empty", title: "Empty" },
];

it("puts each widget in its section in order and drops sections with nothing to show", () => {
  const placed = placeWidgets(sections, [widget("a", "media"), widget("b", "now"), widget("c", "media")]);
  expect(placed.map((entry) => [entry.section.id, entry.widgets.map((item) => item.key)])).toEqual([
    ["now", ["b"]],
    ["media", ["a", "c"]],
  ]);
});

it("a widget without a section, or with one that is not listed, goes to the first section", () => {
  const placed = placeWidgets(sections, [widget("a", null), widget("b", "gone")]);
  expect(placed.map((entry) => [entry.section.id, entry.widgets.map((item) => item.key)])).toEqual([["now", ["a", "b"]]]);
});
