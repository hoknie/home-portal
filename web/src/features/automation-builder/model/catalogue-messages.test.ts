import { expect, it } from "vitest";

import { catalogueSchema, messageKeyOf } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { dictionaries } from "@/shared/i18n";

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);

function has(messages: unknown, key: string) {
  return typeof key.split(".").reduce<unknown>((node, part) => (node as Record<string, unknown> | undefined)?.[part], messages) === "string";
}

it("every event, field and filter of the catalogue has its text in every dictionary", () => {
  const missing = Object.entries(dictionaries).flatMap(([locale, messages]) =>
    catalogue.events.flatMap((event) => {
      const keys = [
        `automationEvents.${messageKeyOf(event.name)}.title`,
        `automationEvents.${messageKeyOf(event.name)}.description`,
        ...event.fields.map((field) => `automationFields.${messageKeyOf(field.name)}`),
        ...event.filters.map((filter) => (filter === "cron" ? "automationBuilder.cron" : `automationBuilder.filters.${filter}`)),
      ];
      return keys.filter((key) => !has(messages, key)).map((key) => `${locale}: ${key}`);
    }),
  );
  expect(missing).toEqual([]);
});

it("every state a status filter accepts has its text", () => {
  expect(catalogue.states.filter((state) => !has(dictionaries.ru, `status.${state}`))).toEqual([]);
});
