import { expect, it } from "vitest";

import { dashboardSchema } from "@/entities/dashboard";
import { apiSamples } from "@/shared/api";

import { fromLayout } from "./draft";
import { placeErrors } from "./errors";

it("puts each server error on the widget or section it names", () => {
  const draft = fromLayout(dashboardSchema.parse(apiSamples.dashboard));
  const placed = placeErrors(
    [
      { field: "widgets[1].settings.latitude", message: "must be between -90 and 90" },
      { field: "sections[1].id", message: "is used by another section" },
      { field: "sections", message: "must hold at least one section" },
    ],
    draft,
    draft,
  );
  expect(placed.widgets.riga).toEqual(["widgets[1].settings.latitude: must be between -90 and 90"]);
  expect(placed.sections.media).toEqual(["sections[1].id: is used by another section"]);
  expect(placed.other).toEqual(["sections: must hold at least one section"]);
});
