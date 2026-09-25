import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { serviceStateSchema, servicesSchema, tlsModeSchema } from "./schema";

it("the services sample parses with every state it carries", () => {
  const parsed = servicesSchema.parse(apiSamples.services);
  expect(parsed.services.map((service) => service.status.state)).toEqual(["up", "down", "unknown"]);
  expect(parsed.services[1].status.last_ok_at).not.toBeNull();
});

it("a state the interface does not know becomes unknown", () => {
  expect(serviceStateSchema.parse("maintenance")).toBe("unknown");
});

it("a published service carries its publication with defaults", () => {
  const parsed = servicesSchema.parse(apiSamples.services);
  expect(parsed.services[0].proxy).toBeNull();
  expect(parsed.services[1].proxy).toEqual({
    host: "nas.example.com",
    upstream: null,
    environments: ["internet"],
    auth: ["internet"],
    tls: null,
    upstream_verify: true,
  });
});

it("a TLS mode the interface does not know becomes acme", () => {
  expect(tlsModeSchema.parse("wildcard")).toBe("acme");
});

