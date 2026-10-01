import { z } from "zod";

import { generated } from "@/shared/api";

export const sessionSchema = generated.session.sessionResponseSchema;

export type Session = z.infer<typeof sessionSchema>;

export const credentialsSchema = z.object({
  name: z.string().trim().min(1, "validation.required"),
  password: z.string().min(1, "validation.required"),
});

export type Credentials = z.infer<typeof credentialsSchema>;
