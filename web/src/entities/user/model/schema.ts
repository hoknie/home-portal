import type { z } from "zod";

import { generated } from "@/shared/api";

export const ADMIN_GROUP = "admin";

export const userSchema = generated.users.userResponseSchema;

export type User = z.infer<typeof userSchema>;

export const usersSchema = generated.users.usersResponseSchema;

export type Users = z.infer<typeof usersSchema>;

export function admins(users: Users) {
  return users.users.filter((user) => user.group === ADMIN_GROUP).length;
}

export function lastAdmin(user: User, users: Users) {
  return user.group === ADMIN_GROUP && admins(users) <= 1;
}

export function deletable(user: User, users: Users) {
  return users.editable && !user.you && users.users.length > 1 && !lastAdmin(user, users);
}
