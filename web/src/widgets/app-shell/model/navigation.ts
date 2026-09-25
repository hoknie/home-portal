import { LayoutDashboard, Network, Server, Waypoints, Webhook, Workflow } from "lucide-react";

import { routes } from "@/shared/config";

export const NAVIGATION = [
  { href: routes.adminServices, label: "services", icon: Server },
  { href: routes.adminLayout, label: "layout", icon: LayoutDashboard },
  { href: routes.adminNetwork, label: "network", icon: Network },
  { href: routes.adminProxy, label: "proxy", icon: Waypoints },
  { href: routes.adminAutomations, label: "automations", icon: Workflow },
  { href: routes.adminWebhooks, label: "webhooks", icon: Webhook },
] as const;

export function isActive(pathname: string, href: string) {
  const normalized = pathname.endsWith("/") ? pathname : `${pathname}/`;
  return href === routes.home ? normalized === routes.home : normalized.startsWith(href);
}
