import { z } from "zod";

export const secretResponseSchema = z.object({ "name": z.string(), "set": z.boolean() });

export type SecretResponse = z.infer<typeof secretResponseSchema>;

export const secretsResponseSchema = z.object({ "secrets": z.array(secretResponseSchema) });

export type SecretsResponse = z.infer<typeof secretsResponseSchema>;

export const schema = secretsResponseSchema;
