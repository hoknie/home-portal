import { Bell, Blocks, FileCode, Globe, House, LayoutDashboard, Network, Route, Server, ShieldCheck, Users, Waypoints, Webhook, Workflow, type LucideIcon } from "lucide-react";

import { CATEGORY_OF, MODULE_CATEGORIES, type ModuleCategory, type ModuleName } from "@/entities/module";
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

export type CategoryLinks = { category: ModuleCategory; links: SectionLink[] };

export const SCRIPTS_CATEGORY: ModuleCategory = "automation";

export function categoryLinks(enabled: ReadonlySet<ModuleName>, editing: boolean, may: MayOpen = EVERY_AREA): CategoryLinks[] {
  const modules = moduleLinks(enabled, may);
  const scripts = editing && may("scripts") ? [SCRIPTS_LINK] : [];
  return MODULE_CATEGORIES.map((category) => ({
    category,
    links: [...modules.filter((link) => CATEGORY_OF[link.module] === category), ...(category === SCRIPTS_CATEGORY ? scripts : [])] as SectionLink[],
  })).filter((group) => group.links.length > 0);
}

export function isActive(pathname: string, href: string) {
  const normalized = pathname.endsWith("/") ? pathname : `${pathname}/`;
  return href === routes.home ? normalized === routes.home : normalized.startsWith(href);
}
