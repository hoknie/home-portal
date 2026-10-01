import type { z } from "zod";

import { generated } from "@/shared/api";

const served = generated.permissions;

export const permissionStateSchema = served.permissionStateSchema;

export const PERMISSION_STATES = permissionStateSchema.options;

export const ADVICES = served.adviceSchema.options;

export const PANES = served.paneSchema.options;

export type PermissionState = z.infer<typeof permissionStateSchema>;

export const permissionSchema = served.permissionResponseSchema;

export type Permission = z.infer<typeof permissionSchema>;

export const permissionsSchema = served.permissionsResponseSchema;

export type Permissions = z.infer<typeof permissionsSchema>;

export type PermissionSubject =
  | { kind: "local-network" | "removable-volumes" | "full-disk-access" }
  | { kind: "folder" | "automation"; name: string };

export function subjectOf(code: string): PermissionSubject {
  const [kind, ...rest] = code.split(":");
  const name = rest.join(":");
  if (kind === "folder" || kind === "automation") {
    return { kind, name };
  }
  if (kind === "local-network" || kind === "removable-volumes") {
    return { kind };
  }
  return { kind: "full-disk-access" };
}

export function anyPending(permissions: Permissions | undefined): boolean {
  return (permissions?.permissions ?? []).some((permission) => permission.state === "pending");
}

export function needsAction(permission: Permission): boolean {
  return permission.state !== "granted" && permission.state !== "not-applicable" && permission.advice !== null;
}
