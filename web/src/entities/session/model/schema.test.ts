import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { allows, canFor, mayOpen } from "./rights";
import { type Session, credentialsSchema, sessionSchema } from "./schema";

const anna = sessionSchema.parse(apiSamples.session);

const admin: Session = { name: "root", group: "admin", admin: true, rights: {} };

const guest: Session = { name: "guest", group: null, admin: false, rights: {} };

it("a right is allowed only when the group holds it, and always for admin", () => {
  expect(allows(anna, "automations", "execute")).toBe(true);
  expect(allows(anna, "automations", "update")).toBe(false);
  expect(allows(guest, "automations", "read")).toBe(false);
  expect(allows(admin, "proxy", "update")).toBe(true);
  expect(allows(undefined, "services", "update")).toBe(false);
  expect(canFor(anna)("automations", "read")).toBe(true);
});

it("pages open with read, services for everyone, and layout with update", () => {
  expect(mayOpen(anna, "automations")).toBe(true);
  expect(mayOpen(anna, "proxy")).toBe(false);
  expect(mayOpen(guest, "services")).toBe(true);
  expect(mayOpen(guest, "modules")).toBe(false);
  expect(mayOpen(anna, "modules")).toBe(false);
  expect(mayOpen(guest, "layout")).toBe(false);
  expect(mayOpen(admin, "layout")).toBe(true);
});

it("credentials need a name and a password", () => {
  const result = credentialsSchema.safeParse({ name: " ", password: "" });
  expect(result.success).toBe(false);
  expect(result.error?.issues.map((issue) => issue.message)).toEqual(["validation.required", "validation.required"]);
});
