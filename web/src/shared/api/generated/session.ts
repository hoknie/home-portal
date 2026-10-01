import { z } from "zod";

export const sessionResponseSchema = z.object({ "admin": z.boolean(), "group": z.string().nullable(), "name": z.string(), "rights": z.record(z.string(), z.array(z.string())) });

export type SessionResponse = z.infer<typeof sessionResponseSchema>;

export const schema = sessionResponseSchema;
