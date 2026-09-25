import { expect, it } from "vitest";

import { servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";

import { countByState, groupServices } from "./grouping";

const services = servicesSchema.parse(apiSamples.services).services;

it("groups by the configured groups, in their order, and only those", () => {
  expect(groupServices(services, ["Media"]).map((group) => [group.name, group.services.map((service) => service.id)])).toEqual([
    ["Media", ["media"]],
  ]);
});

it("without configured groups, named groups come first and the rest last", () => {
  expect(groupServices(services).map((group) => group.name)).toEqual(["Media", null]);
});

it("counts services per state", () => {
  expect(countByState(services)).toEqual({ up: 1, degraded: 0, down: 1, unreadable: 0, unknown: 1 });
});
