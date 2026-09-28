import { Bell, Blocks, Globe, House, LayoutDashboard, Network, Route, Server, Users, Waypoints, Webhook, Workflow, type LucideIcon } from "lucide-react";

import type { ModuleName } from "@/entities/module";
import { routes } from "@/shared/config";

export const MANAGEMENT = [
  { href: routes.home, label: "home", icon: House },
  { href: routes.adminServices, label: "services", icon: Server },
  { href: routes.adminLayout, label: "layout", icon: LayoutDashboard },
  { href: routes.adminNetwork, label: "network", icon: Network },
  { href: routes.adminModules, label: "modules", icon: Blocks },
] as const;

export const MODULE_LINKS = [
  { module: "proxy", href: routes.adminProxy, label: "proxy", icon: Waypoints },
  { module: "dns", href: routes.adminDns, label: "dns", icon: Globe },
  { module: "automations", href: routes.adminAutomations, label: "automations", icon: Workflow },
  { module: "webhooks", href: routes.adminWebhooks, label: "webhooks", icon: Webhook },
  { module: "users", href: routes.adminUsers, label: "users", icon: Users },
  { module: "workflows", href: routes.adminWorkflows, label: "workflows", icon: Route },
  { module: "notifications", href: routes.adminNotifications, label: "notifications", icon: Bell },
] as const satisfies ReadonlyArray<{ module: ModuleName; href: string; label: string; icon: LucideIcon }>;

export function moduleLinks(enabled: ReadonlySet<ModuleName>) {
  return MODULE_LINKS.filter((link) => enabled.has(link.module));
}

export function isActive(pathname: string, href: string) {
  const normalized = pathname.endsWith("/") ? pathname : `${pathname}/`;
  return href === routes.home ? normalized === routes.home : normalized.startsWith(href);
}
