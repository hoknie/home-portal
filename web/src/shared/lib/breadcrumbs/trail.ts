"use client";

import { useTranslations } from "next-intl";

import { routes } from "@/shared/config";

export type TrailItem = { label: string; href?: string; local?: boolean };

export const SECTIONS = {
  services: routes.adminServices,
  layout: routes.adminLayout,
  network: routes.adminNetwork,
  modules: routes.adminModules,
  proxy: routes.adminProxy,
  dns: routes.adminDns,
  automations: routes.adminAutomations,
  webhooks: routes.adminWebhooks,
  users: routes.adminUsers,
  workflows: routes.adminWorkflows,
  notifications: routes.adminNotifications,
  scripts: routes.adminScripts,
  runs: routes.adminRuns,
  permissions: routes.adminPermissions,
} as const;

export type Section = keyof typeof SECTIONS;

export function useTrail() {
  const t = useTranslations("nav");
  const home: TrailItem = { label: t("home"), href: routes.home };
  return {
    home,
    section: (name: Section): TrailItem => ({ label: t(name), href: SECTIONS[name] }),
    of: (...items: TrailItem[]): TrailItem[] => [home, ...items],
  };
}
