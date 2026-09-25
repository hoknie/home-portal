import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { environmentSchema } from "./schema";

it("the environment sample names the visitor's environment and every configured one", () => {
  const parsed = environmentSchema.parse(apiSamples.environment);
  expect(parsed.environment).toBe("local");
  expect(parsed.environments).toEqual(["local", "vpn", "internet"]);
  expect(parsed.detected).toBe("local");
  expect(parsed.switchable).toBe(true);
});
