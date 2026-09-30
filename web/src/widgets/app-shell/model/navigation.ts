import { Bell, Blocks, FileCode, Globe, House, LayoutDashboard, Network, Route, Server, ShieldCheck, Users, Waypoints, Webhook, Workflow, type LucideIcon } from "lucide-react";

import type { ModuleName } from "@/entities/module";
import type { Area } from "@/entities/session";
import { routes } from "@/shared/config";

export type MayOpen = (area: Area) => boolean;

export const MANAGEMENT = [
  { href: routes.home, label: "home", icon: House, area: null },
  { href: routes.adminServices, label: "services", icon: Server, area: "services" },
  { href: routes.adminLayout, label: "layout", icon: LayoutDashboard, area: "layout" },
  { href: routes.adminNetwork, label: "network", icon: Network, area: "network" },
  { href: routes.adminModules, label: "modules", icon: Blocks, area: "modules" },
  { href: routes.adminPermissions, label: "permissions", icon: ShieldCheck, area: "host-permissions" },
] as const satisfies ReadonlyArray<{ href: string; label: string; icon: LucideIcon; area: Area | null }>;

export function managementLinks(may: MayOpen) {
  return MANAGEMENT.filter((link) => link.area === null || may(link.area));
}

export const MODULE_LINKS = [
  { module: "proxy", href: routes.adminProxy, label: "proxy", icon: Waypoints },
  { module: "dns", href: routes.adminDns, label: "dns", icon: Globe },
  { module: "automations", href: routes.adminAutomations, label: "automations", icon: Workflow },
  { module: "webhooks", href: routes.adminWebhooks, label: "webhooks", icon: Webhook },
  { module: "users", href: routes.adminUsers, label: "users", icon: Users },
  { module: "workflows", href: routes.adminWorkflows, label: "workflows", icon: Route },
  { module: "notifications", href: routes.adminNotifications, label: "notifications", icon: Bell },
] as const satisfies ReadonlyArray<{ module: ModuleName & Area; href: string; label: string; icon: LucideIcon }>;

const EVERY_AREA: MayOpen = () => true;

export function moduleLinks(enabled: ReadonlySet<ModuleName>, may: MayOpen = EVERY_AREA) {
  return MODULE_LINKS.filter((link) => enabled.has(link.module) && may(link.module));
}

export const SCRIPTS_LINK = { href: routes.adminScripts, label: "scripts", icon: FileCode } as const;

export type SectionLink = { href: string; label: (typeof MODULE_LINKS)[number]["label"] | typeof SCRIPTS_LINK.label; icon: LucideIcon };

export function sectionLinks(enabled: ReadonlySet<ModuleName>, editing: boolean, may: MayOpen = EVERY_AREA): SectionLink[] {
  return [...moduleLinks(enabled, may), ...(editing && may("scripts") ? [SCRIPTS_LINK] : [])];
}

export function isActive(pathname: string, href: string) {
  const normalized = pathname.endsWith("/") ? pathname : `${pathname}/`;
  return href === routes.home ? normalized === routes.home : normalized.startsWith(href);
}
