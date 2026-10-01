import { z } from "zod";

export const tokenResponseSchema = z.object({ "token": z.string() });

export type TokenResponse = z.infer<typeof tokenResponseSchema>;

export const schema = tokenResponseSchema;
