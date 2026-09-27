import { z } from "zod";

export const userSchema = z.object({ name: z.string(), you: z.boolean() });

export type User = z.infer<typeof userSchema>;

export const usersSchema = z.object({ users: z.array(userSchema), editable: z.boolean() });

export type Users = z.infer<typeof usersSchema>;

export function deletable(user: User, users: Users) {
  return users.editable && !user.you && users.users.length > 1;
}
