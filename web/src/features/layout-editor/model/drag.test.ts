import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import { AREA_KIND, SECTION_KIND, WIDGET_KIND, afterDrop, sectionOfSortable } from "./drag";
import { fromLayout, toRequest, widgetsOf } from "./draft";

const draft = () => fromLayout(dashboardSchema.parse(apiSamples.dashboard));

const keys = (value: ReturnType<typeof draft>, section: string) => widgetsOf(value, section).map((widget) => widget.uid);

it("drops a widget onto another widget's place, in any section", () => {
  const moved = afterDrop(draft(), { id: "riga", kind: WIDGET_KIND, section: "now" }, { id: "#3", kind: WIDGET_KIND, section: "media" });
  expect(keys(moved, "media")).toEqual(["#2", "riga", "#3"]);
});

it("drops a widget onto a section's empty area at its end", () => {
  const moved = afterDrop(draft(), { id: "riga", kind: WIDGET_KIND, section: "now" }, { id: "section:media", kind: AREA_KIND, section: "media" });
  expect(keys(moved, "media")).toEqual(["#2", "#3", "riga"]);
});

it("drops a section in another section's place, taking its widgets along", () => {
  const moved = afterDrop(
    draft(),
    { id: "sort-section:media", kind: SECTION_KIND, section: "media" },
    { id: "sort-section:now", kind: SECTION_KIND, section: "now" },
  );
  expect(moved.sections.map((section) => section.id)).toEqual(["media", "now"]);
  expect(toRequest(moved).widgets.map((widget) => widget.section)).toEqual(["media", "media", "now", "now"]);
});

it("ignores a drop on nothing or on itself", () => {
  const start = draft();
  expect(afterDrop(start, { id: "riga", kind: WIDGET_KIND, section: "now" }, null)).toBe(start);
  expect(afterDrop(start, { id: "riga", kind: WIDGET_KIND, section: "now" }, { id: "riga", kind: WIDGET_KIND, section: "now" })).toBe(start);
});

it("reads the section of a sortable section id", () => {
  expect(sectionOfSortable("sort-section:media")).toBe("media");
  expect(sectionOfSortable("riga")).toBeNull();
});
