import type { Session } from "./schema";

export const AREAS = [
  "services",
  "layout",
  "network",
  "modules",
  "scripts",
  "secrets",
  "host-permissions",
  "portal",
  "proxy",
  "dns",
  "automations",
  "webhooks",
  "users",
  "workflows",
  "notifications",
] as const;

export type Area = (typeof AREAS)[number];

export const ACTIONS = ["read", "create", "update", "delete", "execute"] as const;

export type Action = (typeof ACTIONS)[number];

export type Can = (area: Area, action: Action) => boolean;

const OPEN_TO_EVERYONE: ReadonlySet<Area> = new Set(["services"]);

const OPENED_BY_UPDATE: ReadonlySet<Area> = new Set(["layout"]);

export function allows(session: Session | undefined, area: Area, action: Action): boolean {
  if (!session) {
    return false;
  }
  return session.admin || (session.rights[area]?.includes(action) ?? false);
}

export function mayOpen(session: Session | undefined, area: Area): boolean {
  if (!session) {
    return false;
  }
  if (OPEN_TO_EVERYONE.has(area)) {
    return true;
  }
  return allows(session, area, OPENED_BY_UPDATE.has(area) ? "update" : "read");
}

export function canFor(session: Session | undefined): Can {
  return (area, action) => allows(session, area, action);
}
