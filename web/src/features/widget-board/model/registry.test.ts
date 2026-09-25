import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";
import { dictionaries } from "@/shared/i18n";

import { WIDGETS } from "./registry";

it("the registry covers exactly the widget kinds the portal serves data for", () => {
  const backed = Object.entries(WIDGETS)
    .filter(([, entry]) => entry.data)
    .map(([kind]) => kind)
    .sort();
  expect(backed).toEqual([...apiSamples.widgetKinds.kinds].sort());
});

it("every weather condition the portal reports has a Russian name", () => {
  const conditions = dictionaries.ru.conditions as Record<string, string>;
  expect(apiSamples.widgetKinds.conditions.filter((condition) => !conditions[condition])).toEqual([]);
});
