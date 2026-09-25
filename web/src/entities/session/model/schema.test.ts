import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { credentialsSchema, sessionSchema } from "./schema";

it("the session sample parses", () => {
  expect(sessionSchema.parse(apiSamples.session)).toEqual({ name: "admin" });
});

it("credentials need a name and a password", () => {
  const result = credentialsSchema.safeParse({ name: " ", password: "" });
  expect(result.success).toBe(false);
  expect(result.error?.issues.map((issue) => issue.message)).toEqual(["validation.required", "validation.required"]);
});
