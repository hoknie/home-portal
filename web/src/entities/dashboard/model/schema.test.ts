import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { dashboardSchema } from "./schema";

it("a size from a newer portal reads as full", () => {
  const future = { ...apiSamples.dashboard, widgets: [{ ...apiSamples.dashboard.widgets[0], size: "huge" }] };
  expect(dashboardSchema.parse(future).widgets[0].size).toBe("full");
});
