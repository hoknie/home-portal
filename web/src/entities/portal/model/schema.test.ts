import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { portalSchema } from "./schema";

it("the public portal sample parses: the environment, its services and its widgets", () => {
  const portal = portalSchema.parse(apiSamples.publicPortal);
  expect(portal.environment).toBe("local");
  expect(portal.detected).toBe("local");
  expect(portal.switchable).toBe(true);
  expect(portal.environments).toEqual(["local", "vpn", "internet"]);
  expect(portal.services[0].icon).toBe("/api/public/icons/media");
  expect(portal.services[0].status?.state).toBe("up");
  expect(portal.widgets[0]).toMatchObject({ type: "weather", id: "riga", section: "now", size: "third" });
  expect(portal.sections).toEqual([{ id: "now", title: "Сейчас" }]);
});

it("a service the portal hides the status of still parses", () => {
  const portal = portalSchema.parse({
    ...apiSamples.publicPortal,
    services: [{ ...apiSamples.publicPortal.services[0], status: null }],
  });
  expect(portal.services[0].status).toBeNull();
});
