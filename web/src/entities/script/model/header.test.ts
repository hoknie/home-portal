import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { parseHeader } from "./header";
import { scriptHeaderSchema } from "./schema";

type Case = { name: string; text: string; header: unknown };

const cases = (apiSamples.scriptHeaders as { cases: Case[] }).cases;

describe("the script header fixtures shared with the server", () => {
  it.each(cases.map((entry) => [entry.name, entry] as const))("%s reads as the server reads it", (_, entry) => {
    expect(parseHeader(entry.text)).toEqual(scriptHeaderSchema.parse(entry.header));
  });

  it("stops after sixty-four lines", () => {
    const text = `${"#\n".repeat(64)}# @arg late\n`;
    expect(parseHeader(text).arguments).toEqual([]);
  });

  it("keeps at most thirty-two arguments and names the rest", () => {
    const text = Array.from({ length: 40 }, (_, index) => `# @arg --o${index}`).join("\n");
    const header = parseHeader(text);
    expect(header.arguments).toHaveLength(32);
    expect(header.problems).toHaveLength(8);
  });
});
