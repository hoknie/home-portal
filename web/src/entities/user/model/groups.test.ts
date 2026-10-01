import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { columnState, givable, groupsSchema, matrixState, rowState, withAction, withColumn, withEverything, withRow, within } from "./groups";

const groups = groupsSchema.parse(apiSamples.groups);

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

const matrix = [
  { area: "services", actions: ["create", "update", "delete"] },
  { area: "automations", actions: ["read", "create", "update", "delete", "execute"] },
  { area: "secrets", actions: ["read"] },
];

it("a column gives its action to every area that offers it and skips the others", () => {
  const read = withColumn({}, "read", true, matrix);
  expect(read).toEqual({ automations: ["read"], secrets: ["read"] });
  expect(columnState(read, "read", matrix)).toBe("all");
  expect(withColumn(read, "read", false, matrix)).toEqual({});
});

it("a row takes only the area's own actions, in their order, and clearing it drops the area", () => {
  const rights = withRow({ secrets: ["read"] }, "automations", true, matrix);
  expect(rights.automations).toEqual(["read", "create", "update", "delete", "execute"]);
  expect(rowState(rights, "automations", matrix)).toBe("all");
  expect(columnState(rights, "update", matrix)).toBe("some");
  expect(withRow(rights, "automations", false, matrix)).toEqual({ secrets: ["read"] });
});

it("selecting everything equals ticking every cell by hand, and clearing it leaves no rights", () => {
  const byHand = matrix.flatMap((row) => row.actions.map((action) => [row.area, action] as const)).reduce((next, [area, action]) => withAction(next, area, action, true, matrix), {});
  expect(withEverything(true, matrix)).toEqual(byHand);
  expect(matrixState(byHand, matrix)).toBe("all");
  expect(withEverything(false, matrix)).toEqual({});
});

it("coverage is none, some or all", () => {
  expect(rowState({}, "services", matrix)).toBe("none");
  expect(rowState({ services: ["update"] }, "services", matrix)).toBe("some");
  expect(matrixState({ services: ["update"] }, matrix)).toBe("some");
  expect(matrixState({}, matrix)).toBe("none");
});
