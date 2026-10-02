import { expect, it } from "vitest";

import { DEFAULT_APPEARANCE, DEFAULT_SECTION_APPEARANCE } from "@/shared/api";

import { type GridWidget, placeWidgets } from "./layout";

const widget = (key: string, section: string | null): GridWidget => ({
  key,
  type: "status-summary",
  id: null,
  title: null,
  settings: {},
  section,
  width: 12,
  height: "auto",
  column: null,
  row: null,
  appearance: DEFAULT_APPEARANCE,
});

const sections = [
  { id: "now", title: "Now", appearance: DEFAULT_SECTION_APPEARANCE },
  { id: "media", title: "Media", appearance: DEFAULT_SECTION_APPEARANCE },
  { id: "empty", title: "Empty", appearance: DEFAULT_SECTION_APPEARANCE },
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

