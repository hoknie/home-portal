import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { proxySchema, proxyTlsModeSchema, usesInternal } from "./schema";

describe("proxy", () => {
  it("a route on the internal authority means the internal certificate is in use", () => {
    const parsed = proxySchema.parse(apiSamples.proxy);
    expect(usesInternal(parsed)).toBe(true);
    expect(usesInternal({ ...parsed, routes: parsed.routes.filter((route) => route.tls !== "internal") })).toBe(false);
  });

  it("a TLS mode the interface does not know becomes acme", () => {
    expect(proxyTlsModeSchema.parse("wildcard")).toBe("acme");
  });
});
