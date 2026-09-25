import { describe, expect, it } from "vitest";

import { locales } from "./config";
import { dictionaries } from "./messages";

function keys(value: unknown, prefix = ""): string[] {
  if (typeof value !== "object" || value === null) {
    return [prefix];
  }
  return Object.entries(value).flatMap(([key, child]) => keys(child, prefix ? `${prefix}.${key}` : key));
}

describe("dictionaries", () => {
  it("every language has exactly the keys of the Russian dictionary", () => {
    const reference = keys(dictionaries.ru).sort();
    for (const [locale, messages] of Object.entries(dictionaries)) {
      expect(keys(messages).sort(), locale).toEqual(reference);
    }
  });

  it("no message is empty", () => {
    const empty = keys(dictionaries.ru).filter((key) => {
      const value = key.split(".").reduce<unknown>((node, part) => (node as Record<string, unknown>)[part], dictionaries.ru);
      return typeof value === "string" && value.trim() === "";
    });
    expect(empty).toEqual([]);
  });

  it("every dictionary names each language in that language", () => {
    for (const messages of Object.values(dictionaries)) {
      expect(messages.language.names).toEqual({ en: "English", ru: "Русский", es: "Español" });
    }
  });

  it("there is one dictionary per supported language", () => {
    expect(Object.keys(dictionaries).sort()).toEqual([...locales].sort());
  });

  it("every message keeps the placeholders of its Russian original in every language", () => {
    const placeholders = (text: string) => [...new Set([...text.matchAll(/\{\s*([A-Za-z]+)/g)].map((match) => match[1]))].sort();
    const at = (messages: unknown, key: string) =>
      key.split(".").reduce<unknown>((node, part) => (node as Record<string, unknown>)[part], messages);
    const differing = Object.entries(dictionaries).flatMap(([locale, messages]) =>
      keys(dictionaries.ru)
        .filter((key) => {
          const original = at(dictionaries.ru, key);
          const translated = at(messages, key);
          return typeof original === "string" && typeof translated === "string" && placeholders(original).join() !== placeholders(translated).join();
        })
        .map((key) => `${locale}: ${key}`),
    );
    expect(differing).toEqual([]);
  });
});
