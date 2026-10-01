import { z } from "zod";

export const environmentSchema = z.string();

export type Environment = z.infer<typeof environmentSchema>;

export const environmentResponseSchema = z.object({ "detected": environmentSchema, "environment": environmentSchema, "environments": z.array(environmentSchema), "switchable": z.boolean() });

export type EnvironmentResponse = z.infer<typeof environmentResponseSchema>;

export const schema = environmentResponseSchema;
