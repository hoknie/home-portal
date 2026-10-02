import { afterEach, expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import { fromLayout, updateWidget } from "./draft";
import { dropDraft, keepDraft, keptDraft, resizeKept, resizedLayout } from "./kept-draft";

afterEach(dropDraft);

it("keeps an unsaved arrangement for the same layout and drops it for another", () => {
  const base = fromLayout(dashboardSchema.parse(apiSamples.dashboard));
  const moved = updateWidget(base, "#0", { width: 4 });
  keepDraft(base, moved);
  expect(keptDraft(base)).toEqual(moved);
  expect(keptDraft(moved)).toBeNull();
  dropDraft();
  expect(keptDraft(base)).toBeNull();
});

it("a size set in the builder lands on its place, in the saved layout and in the kept arrangement", () => {
  const layout = dashboardSchema.parse(apiSamples.dashboard);
  const base = fromLayout(layout);
  expect(resizedLayout(layout, "#1", { width: 6, height: 2 }).widgets[1]).toMatchObject({ key: "#1", width: 6, height: 2 });
  keepDraft(base, updateWidget(base, "#0", { width: 4 }));
  resizeKept("#1", { width: 6, height: "auto" }, null);
  const kept = keptDraft(base);
  expect(kept?.widgets[0]).toMatchObject({ width: 4 });
  expect(kept?.widgets[1]).toMatchObject({ width: 6, height: "auto" });
});
