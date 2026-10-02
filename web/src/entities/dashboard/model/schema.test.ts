import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { dashboardSchema } from "./schema";

it("a look from a newer portal reads as the default of each setting", () => {
  const widget = apiSamples.dashboard.widgets[0];
  const future = { ...apiSamples.dashboard, widgets: [{ ...widget, appearance: { ...widget.appearance, surface: "glass", accent: "gold" } }] };
  const parsed = dashboardSchema.parse(future).widgets[0].appearance;
  expect(parsed.surface).toBe("card");
  expect(parsed.accent).toBe("neutral");
});

it("a widget carries its width, height and look", () => {
  const [summary, weather] = dashboardSchema.parse(apiSamples.dashboard).widgets;
  expect(summary).toMatchObject({ width: 8, height: "auto", appearance: { surface: "plain", title: "hidden" } });
  expect(weather).toMatchObject({ width: 4, height: 3, appearance: { surface: "tinted", accent: "blue" } });
});
