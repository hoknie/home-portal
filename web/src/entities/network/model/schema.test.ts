import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { networkFormOf, networkFormSchema, networkRequestOf } from "./form";
import { networkSchema } from "./schema";

function messages(form: unknown) {
  const result = networkFormSchema.safeParse(form);
  return result.success ? {} : Object.fromEntries(result.error.issues.map((issue) => [issue.path.join("."), issue.message]));
}

describe("network", () => {
  it("the network sample parses", () => {
    const parsed = networkSchema.parse(apiSamples.network);
    expect(parsed.restart_required).toBe(true);
    expect(parsed.interfaces[0].name).toBe("en0");
  });

  it("the form round-trips the configured settings", () => {
    const configured = networkSchema.parse(apiSamples.network).configured;
    expect(networkRequestOf(networkFormOf(configured))).toEqual(configured);
  });

  it("names every invalid field with a message key", () => {
    expect(messages({ address: "nowhere", port: 70000, public_url: "ftp://x", trusted_proxies: "10.0.0.0/8\n10.0.0.0/33" })).toEqual({
      address: "validation.address",
      port: "validation.port",
      public_url: "validation.publicUrl",
      trusted_proxies: "validation.trustedProxy",
    });
  });

  it("accepts addresses and ranges of both families", () => {
    expect(messages({ address: "::", port: 8080, public_url: "", trusted_proxies: "192.168.1.1, fd00::/8" })).toEqual({});
  });
});
