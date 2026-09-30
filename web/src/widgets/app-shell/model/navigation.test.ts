import { expect, it } from "vitest";

import { type Session, mayOpen } from "@/entities/session";

import { MANAGEMENT, MODULE_LINKS, isActive, managementLinks, moduleLinks, sectionLinks } from "./navigation";

it("marks a management section on its own page and its subpages only", () => {
  expect(isActive("/", "/")).toBe(true);
  expect(isActive("/admin/services/", "/")).toBe(false);
  expect(isActive("/admin/services", "/admin/services/")).toBe(true);
  expect(isActive("/admin/network/", "/admin/layout/")).toBe(false);
});

it("the management section always offers home, services, layout, network and modules", () => {
  expect(MANAGEMENT.map((item) => item.href)).toEqual(["/", "/admin/services/", "/admin/layout/", "/admin/network/", "/admin/modules/", "/admin/permissions/"]);
});

it("the modules section lists proxy, dns, automations, webhooks, users, workflows and notifications in that order", () => {
  expect(MODULE_LINKS.map((item) => item.href)).toEqual(["/admin/proxy/", "/admin/dns/", "/admin/automations/", "/admin/webhooks/", "/admin/users/", "/admin/workflows/", "/admin/notifications/"]);
});

it("only enabled modules are listed", () => {
  expect(moduleLinks(new Set(["automations", "proxy"])).map((item) => item.module)).toEqual(["proxy", "automations"]);
  expect(moduleLinks(new Set())).toEqual([]);
});

it("scripts follow the modules while editing is on, whatever modules are switched", () => {
  expect(sectionLinks(new Set(["workflows"]), true).map((item) => item.href)).toEqual(["/admin/workflows/", "/admin/scripts/"]);
  expect(sectionLinks(new Set(), true).map((item) => item.label)).toEqual(["scripts"]);
  expect(sectionLinks(new Set(["workflows"]), false).map((item) => item.label)).toEqual(["workflows"]);
});

const guest: Session = { name: "guest", group: null, admin: false, rights: {} };

it("the menu of a guest lists only home and services, with no modules section", () => {
  const may = (area: Parameters<typeof mayOpen>[1]) => mayOpen(guest, area);
  expect(managementLinks(may).map((item) => item.label)).toEqual(["home", "services"]);
  const every = new Set(MODULE_LINKS.map((link) => link.module));
  expect(sectionLinks(every, true, may)).toEqual([]);
});

it("a module appears only with its read right, and scripts only with scripts read", () => {
  const family: Session = { name: "anna", group: "family", admin: false, rights: { automations: ["read"], layout: ["update"] } };
  const may = (area: Parameters<typeof mayOpen>[1]) => mayOpen(family, area);
  expect(managementLinks(may).map((item) => item.label)).toEqual(["home", "services", "layout"]);
  expect(sectionLinks(new Set(["automations", "proxy"]), true, may).map((item) => item.label)).toEqual(["automations"]);
});
