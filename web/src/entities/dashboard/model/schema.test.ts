import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { dashboardSchema } from "./schema";

it("the dashboard sample parses, unknown widget types included", () => {
  const types = dashboardSchema.parse(apiSamples.dashboard).widgets.map((widget) => widget.type);
  expect(types).toEqual(["status-summary", "weather", "services", "services"]);
});

it("the dashboard sample carries sections, keys, sections and sizes of its widgets", () => {
  const dashboard = dashboardSchema.parse(apiSamples.dashboard);
  expect(dashboard.sections).toEqual([
    { id: "now", title: "Now" },
    { id: "media", title: "Media" },
  ]);
  expect(dashboard.widgets.map((widget) => [widget.key, widget.section, widget.size])).toEqual([
    ["#0", "now", "two-thirds"],
    ["riga", "now", "third"],
    ["#2", "media", "half"],
    ["#3", "media", "half"],
  ]);
  expect(dashboard.widgets[2].environments).toEqual(["local"]);
});

it("a size from a newer portal reads as full", () => {
  const future = { ...apiSamples.dashboard, widgets: [{ ...apiSamples.dashboard.widgets[0], size: "huge" }] };
  expect(dashboardSchema.parse(future).widgets[0].size).toBe("full");
});
