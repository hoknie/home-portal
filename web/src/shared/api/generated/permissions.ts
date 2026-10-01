import { z } from "zod";

export const ownerKindSchema = z.enum(["terminal", "binary"]);

export type OwnerKind = z.infer<typeof ownerKindSchema>;

export const ownerResponseSchema = z.object({ "kind": ownerKindSchema, "name": z.string() });

export type OwnerResponse = z.infer<typeof ownerResponseSchema>;

export const adviceSchema = z.enum(["allow-in-settings", "answer-the-prompt", "connect-a-volume", "grant-full-disk-access", "application-not-found", "check-failed"]);

export type Advice = z.infer<typeof adviceSchema>;

export const paneSchema = z.enum(["local-network", "files-and-folders", "automation", "full-disk-access"]);

export type Pane = z.infer<typeof paneSchema>;

export const permissionStateSchema = z.enum(["granted", "denied", "pending", "not-applicable", "unknown"]);

export type PermissionState = z.infer<typeof permissionStateSchema>;

export const permissionResponseSchema = z.object({ "advice": adviceSchema.nullable(), "code": z.string(), "learned_at": z.string().nullable(), "pane": paneSchema, "state": permissionStateSchema });

export type PermissionResponse = z.infer<typeof permissionResponseSchema>;

export const permissionsResponseSchema = z.object({ "owner": ownerResponseSchema, "permissions": z.array(permissionResponseSchema), "platform": z.string() });

export type PermissionsResponse = z.infer<typeof permissionsResponseSchema>;

export const schema = permissionsResponseSchema;
