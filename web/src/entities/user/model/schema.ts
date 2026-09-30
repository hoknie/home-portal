import { z } from "zod";

export const ADMIN_GROUP = "admin";

export const userSchema = z.object({ name: z.string(), group: z.string().nullable(), you: z.boolean() });

export type User = z.infer<typeof userSchema>;

export const usersSchema = z.object({ users: z.array(userSchema), editable: z.boolean() });

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
