import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import { SECTION_KIND, afterDrop } from "./drag";
import { fromLayout } from "./draft";

const draft = () => fromLayout(dashboardSchema.parse(apiSamples.dashboard));

it("dropping a section on another swaps them and leaves widgets with their sections", () => {
  const moved = afterDrop(draft(), { id: "sort-section:media", kind: SECTION_KIND, section: "media" }, { id: "sort-section:now", kind: SECTION_KIND, section: "now" });
  expect(moved.sections.map((section) => section.id)).toEqual(["media", "now"]);
  expect(moved.widgets[0].section).toBe("media");
});

it("a drop of anything else, or onto nothing, changes nothing", () => {
  const loaded = draft();
  expect(afterDrop(loaded, { id: "x", kind: "widget", section: "now" }, { id: "y", kind: SECTION_KIND, section: "media" })).toBe(loaded);
  expect(afterDrop(loaded, { id: "sort-section:now", kind: SECTION_KIND, section: "now" }, null)).toBe(loaded);
});
