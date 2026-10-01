import { expect, it } from "vitest";

import { apiSamples } from "../samples";
import * as generated from "./index";

it("every sample written from a typed serializer parses with the schema generated from that type", () => {
  const samples: Record<string, unknown> = apiSamples;
  const names = Object.keys(generated);
  expect(names.length).toBeGreaterThan(0);
  for (const name of names) {
    expect(samples[name], name).toBeDefined();
    const parsed = generated[name as keyof typeof generated].schema.safeParse(samples[name]);
    expect(parsed.error, name).toBeUndefined();
  }
});
