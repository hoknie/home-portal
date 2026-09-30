import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { givable, groupsSchema, withAction, within } from "./groups";

const groups = groupsSchema.parse(apiSamples.groups);

it("the groups sample puts admin first and answers the matrix", () => {
  expect(groups.groups.map((group) => [group.name, group.builtin])).toEqual([
    ["admin", true],
    ["family", false],
    ["guests", false],
  ]);
  expect(groups.matrix.find((row) => row.area === "layout")?.actions).toEqual(["update"]);
});

it("rights are within others when every action is held", () => {
  expect(within({ automations: ["read"] }, { automations: ["read", "execute"] })).toBe(true);
  expect(within({ proxy: ["update"] }, { automations: ["read"] })).toBe(false);
  expect(within({}, {})).toBe(true);
});

it("an admin may give every group and anyone else only groups within their rights", () => {
  expect(givable(groups, {}, true)).toEqual(["admin", "family", "guests"]);
  expect(givable(groups, { automations: ["read", "execute"], services: ["update"] }, false)).toEqual(["family", "guests"]);
  expect(givable(groups, {}, false)).toEqual(["guests"]);
});

it("ticking an action keeps the matrix order and unticking the last drops the area", () => {
  const added = withAction({ automations: ["execute"] }, "automations", "read", true, groups.matrix);
  expect(added).toEqual({ automations: ["read", "execute"] });
  expect(withAction({ layout: ["update"] }, "layout", "update", false, groups.matrix)).toEqual({});
});
