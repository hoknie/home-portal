import { z } from "zod";

export const USER_NAME_MAX = 64;
export const PASSWORD_MIN = 8;
export const PASSWORD_MAX = 1024;

const password = z.string().min(PASSWORD_MIN, "validation.userPassword").max(PASSWORD_MAX, "validation.userPassword");

export const newUserFormSchema = z
  .object({
    name: z
      .string()
      .trim()
      .min(1, "validation.userName")
      .max(USER_NAME_MAX, "validation.userName")
      .refine((name) => !/\p{Cc}/u.test(name), "validation.userName"),
    password,
    repeat: z.string(),
  })
  .refine((form) => form.password === form.repeat, { path: ["repeat"], message: "validation.userPasswordsDiffer" });

export type NewUserForm = z.infer<typeof newUserFormSchema>;

export const passwordFormSchema = z
  .object({ password, repeat: z.string() })
  .refine((form) => form.password === form.repeat, { path: ["repeat"], message: "validation.userPasswordsDiffer" });

export type PasswordForm = z.infer<typeof passwordFormSchema>;
