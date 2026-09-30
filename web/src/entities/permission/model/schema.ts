import { z } from "zod";

export const PERMISSION_STATES = ["granted", "denied", "pending", "not-applicable", "unknown"] as const;

export const ADVICES = [
  "allow-in-settings",
  "answer-the-prompt",
  "connect-a-volume",
  "grant-full-disk-access",
  "application-not-found",
  "check-failed",
] as const;

export const PANES = ["local-network", "files-and-folders", "automation", "full-disk-access"] as const;

export const permissionStateSchema = z.enum(PERMISSION_STATES);

export type PermissionState = z.infer<typeof permissionStateSchema>;

export const permissionSchema = z.object({
  code: z.string(),
  state: permissionStateSchema,
  learned_at: z.string().nullable(),
  advice: z.enum(ADVICES).nullable(),
  pane: z.enum(PANES),
});

export type Permission = z.infer<typeof permissionSchema>;

export const permissionsSchema = z.object({
  platform: z.string(),
  owner: z.object({ kind: z.enum(["terminal", "binary"]), name: z.string() }),
  permissions: z.array(permissionSchema),
});

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
