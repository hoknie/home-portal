import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import { addSection, fromLayout, placeWidget, removeSection, removeWidget, renameSection, sameDraft, toRequest, updateWidget, widgetsOf } from "./draft";

const draft = () => fromLayout(dashboardSchema.parse(apiSamples.dashboard));

it("reads each place with the library widget it names, its section, position and size", () => {
  const loaded = draft();
  expect(loaded.widgets.map((widget) => [widget.key, widget.widget, widget.section])).toEqual([
    ["#0", "status-summary", "now"],
    ["#1", "riga", "now"],
    ["#2", "services", "media"],
    ["#3", "services-2", "media"],
  ]);
  expect(loaded.widgets[1]).toMatchObject({ column: 9, row: 1, width: 4, height: 3 });
  expect(toRequest(loaded).widgets[1]).toEqual({ key: "#1", widget: "riga", section: "now", column: 9, row: 1, width: 4, height: 3 });
});

it("places a library widget at the end of a section without a position, and removes a place", () => {
  const [placed, uid] = placeWidget(draft(), "riga", "media");
  expect(widgetsOf(placed, "media").at(-1)).toMatchObject({ uid, key: null, widget: "riga", column: null, row: null, width: 12 });
  expect(sameDraft(removeWidget(placed, uid), draft())).toBe(true);
});

it("changes a size, and manages sections", () => {
  const resized = updateWidget(draft(), "#0", { width: 6, height: 2 });
  expect(resized.widgets[0]).toMatchObject({ width: 6, height: 2 });
  const sectioned = renameSection(addSection(draft(), null), "section-3", "Later");
  expect(sectioned.sections.at(-1)).toMatchObject({ id: "section-3", title: "Later" });
  expect(removeSection(sectioned, "section-3").sections).toHaveLength(2);
  expect(removeSection(sectioned, "now").sections).toHaveLength(3);
});
