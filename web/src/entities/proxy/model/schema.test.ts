import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { proxySchema, proxyTlsModeSchema, usesInternal } from "./schema";

describe("proxy", () => {
  it("the proxy sample parses with the portal first", () => {
    const parsed = proxySchema.parse(apiSamples.proxy);
    expect(parsed.in_sync).toBe(true);
    expect(parsed.routes.map((route) => route.service)).toEqual([null, "nas", "media"]);
    expect(usesInternal(parsed)).toBe(true);
    expect(parsed.caddy).toMatchObject({ managed: true, installed: "2.11.4", download: { state: "idle", error: null } });
    expect(parsed.caddy).toMatchObject({
      version: "latest",
      release_url: "https://api.github.com/repos/caddyserver/caddy/releases/latest",
      platform: "mac_arm64",
      platform_error: null,
    });
    expect(parsed.caddy.installed_from).toMatch(/caddy_2\.11\.4_mac_arm64\.tar\.gz$/);
  });

  it("a TLS mode the interface does not know becomes acme", () => {
    expect(proxyTlsModeSchema.parse("wildcard")).toBe("acme");
  });
});
