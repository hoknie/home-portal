import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { portalSchema } from "./schema";

it("a service the portal hides the status of still parses", () => {
  const portal = portalSchema.parse({
    ...apiSamples.publicPortal,
    services: [{ ...apiSamples.publicPortal.services[0], status: null }],
  });
  expect(portal.services[0].status).toBeNull();
});
