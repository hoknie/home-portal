import type { Area } from "@/entities/session";
import { routes } from "@/shared/config";

import { isActive } from "./navigation";

export const PAGE_AREAS: ReadonlyArray<{ href: string; area: Area }> = [
  { href: routes.adminServices, area: "services" },
  { href: routes.adminLayout, area: "layout" },
  { href: routes.adminNetwork, area: "network" },
  { href: routes.adminModules, area: "modules" },
  { href: routes.adminPermissions, area: "host-permissions" },
  { href: routes.adminProxy, area: "proxy" },
  { href: routes.adminDns, area: "dns" },
  { href: routes.adminAutomations, area: "automations" },
  { href: routes.adminWebhooks, area: "webhooks" },
  { href: routes.adminUsers, area: "users" },
  { href: routes.adminWorkflows, area: "workflows" },
  { href: routes.adminNotifications, area: "notifications" },
  { href: routes.adminScripts, area: "scripts" },
];

export function areaOfPage(pathname: string): Area | null {
  return PAGE_AREAS.find((page) => isActive(pathname, page.href))?.area ?? null;
}
