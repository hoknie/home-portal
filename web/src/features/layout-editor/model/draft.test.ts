import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import {
  addSection,
  addWidget,
  fromLayout,
  moveSection,
  moveWidget,
  removeSection,
  removeWidget,
  renameSection,
  sameDraft,
  toRequest,
  updateWidget,
  widgetsOf,
} from "./draft";

const draft = () => fromLayout(dashboardSchema.parse(apiSamples.dashboard));

const keys = (value: ReturnType<typeof draft>, section: string) => widgetsOf(value, section).map((widget) => widget.uid);

it("reads the layout as sections holding their widgets in order", () => {
  expect(keys(draft(), "now")).toEqual(["#0", "riga"]);
  expect(keys(draft(), "media")).toEqual(["#2", "#3"]);
});

it("moves a widget within a section and into another one", () => {
  const within = moveWidget(draft(), "riga", "now", 0);
  expect(keys(within, "now")).toEqual(["riga", "#0"]);
  const across = moveWidget(draft(), "riga", "media", 1);
  expect(keys(across, "now")).toEqual(["#0"]);
  expect(keys(across, "media")).toEqual(["#2", "riga", "#3"]);
  expect(toRequest(across).widgets.map((widget) => widget.key)).toEqual(["#0", "#2", "riga", "#3"]);
});

it("changes size and section, adds and removes widgets", () => {
  let edited = updateWidget(draft(), "#0", { size: "half", section: "media" });
  expect(keys(edited, "media")).toEqual(["#2", "#3", "#0"]);
  expect(widgetsOf(edited, "media")[2].size).toBe("half");
  edited = addWidget(edited, "weather", "now");
  const added = widgetsOf(edited, "now").at(-1);
  expect(added).toMatchObject({ type: "weather", key: null, id: null, size: "full" });
  edited = removeWidget(edited, "riga");
  expect(edited.widgets.some((widget) => widget.uid === "riga")).toBe(false);
});

it("adds, renames, moves and deletes only empty sections", () => {
  let edited = addSection(draft(), "Позже");
  expect(edited.sections.map((section) => section.id)).toEqual(["now", "media", "section-3"]);
  edited = renameSection(edited, "section-3", "  Вечером ");
  expect(edited.sections[2].title).toBe("Вечером");
  edited = moveSection(edited, "section-3", -2);
  expect(edited.sections.map((section) => section.id)).toEqual(["section-3", "now", "media"]);
  expect(toRequest(edited).widgets.map((widget) => widget.section)).toEqual(["now", "now", "media", "media"]);
  expect(removeSection(edited, "now")).toBe(edited);
  expect(removeSection(edited, "section-3").sections).toHaveLength(2);
});

it("knows when nothing changed", () => {
  expect(sameDraft(draft(), draft())).toBe(true);
  expect(sameDraft(draft(), updateWidget(draft(), "#0", { title: "Сводка" }))).toBe(false);
});
