import { z } from "zod";

export const sessionSchema = z.object({ name: z.string() });

export type Session = z.infer<typeof sessionSchema>;

export const credentialsSchema = z.object({
  name: z.string().trim().min(1, "validation.required"),
  password: z.string().min(1, "validation.required"),
});

export type Credentials = z.infer<typeof credentialsSchema>;
