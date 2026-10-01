import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { PENDING_REFRESH_MILLISECONDS, pendingRefresh } from "./queries";
import { anyPending, needsAction, permissionsSchema, subjectOf } from "./schema";

it("a list with a permission awaiting an answer is pending", () => {
  const permissions = permissionsSchema.parse(apiSamples.permissions);
  expect(anyPending(permissions)).toBe(true);
  expect(anyPending({ ...permissions, permissions: permissions.permissions.filter((permission) => permission.state !== "pending") })).toBe(false);
});

it("a code names its folder or application after the colon", () => {
  expect(subjectOf("folder:Documents")).toEqual({ kind: "folder", name: "Documents" });
  expect(subjectOf("automation:System Events")).toEqual({ kind: "automation", name: "System Events" });
  expect(subjectOf("local-network")).toEqual({ kind: "local-network" });
  expect(subjectOf("full-disk-access")).toEqual({ kind: "full-disk-access" });
});

it("only a permission that is not granted and has advice needs action", () => {
  const [granted, denied] = permissionsSchema.parse(apiSamples.permissions).permissions;
  expect(needsAction(granted)).toBe(false);
  expect(needsAction(denied)).toBe(true);
});

it("the list is refreshed every two seconds only while an answer is awaited", () => {
  const permissions = permissionsSchema.parse(structuredClone(apiSamples.permissions));
  expect(pendingRefresh(permissions)).toBe(PENDING_REFRESH_MILLISECONDS);
  permissions.permissions = permissions.permissions.filter((permission) => permission.state !== "pending");
  expect(pendingRefresh(permissions)).toBe(false);
  expect(pendingRefresh(undefined)).toBe(false);
});
