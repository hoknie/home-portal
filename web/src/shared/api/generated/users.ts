import { z } from "zod";

export const userResponseSchema = z.object({ "group": z.string().nullable(), "name": z.string(), "you": z.boolean() });

export type UserResponse = z.infer<typeof userResponseSchema>;

export const usersResponseSchema = z.object({ "editable": z.boolean(), "users": z.array(userResponseSchema) });

export type UsersResponse = z.infer<typeof usersResponseSchema>;

export const schema = usersResponseSchema;
