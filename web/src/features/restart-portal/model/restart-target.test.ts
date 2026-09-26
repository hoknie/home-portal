import { describe, expect, it } from "vitest";

import { type Network, networkSchema } from "@/entities/network";
import { apiSamples } from "@/shared/api";

import { restartTarget } from "./restart-target";

const direct = { origin: "http://nas.lan:8080", protocol: "http:", hostname: "nas.lan", port: "8080" };

function network(change: Partial<Network>): Network {
  const sample = networkSchema.parse(apiSamples.network);
  return {
    ...sample,
    effective: { address: "0.0.0.0:8080", overridden: false },
    configured: { ...sample.configured, port: 9090 },
    restart_required: true,
    ...change,
  };
}

describe("restartTarget", () => {
  it("stays on the current origin when nothing about the address changes", () => {
    expect(restartTarget(direct)).toBe("http://nas.lan:8080");
    expect(restartTarget(direct, network({ restart_required: false }))).toBe("http://nas.lan:8080");
  });

  it("follows a configured port on the same host when the browser talks to the portal directly", () => {
    expect(restartTarget(direct, network({}))).toBe("http://nas.lan:9090");
  });

  it("stays when the variable overrides the address or a proxy stands in front", () => {
    expect(restartTarget(direct, network({ effective: { address: "0.0.0.0:8080", overridden: true } }))).toBe("http://nas.lan:8080");
    const proxied = { origin: "https://portal.home.lan", protocol: "https:", hostname: "portal.home.lan", port: "" };
    expect(restartTarget(proxied, network({}))).toBe("https://portal.home.lan");
  });
});
