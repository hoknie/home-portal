import { z } from "zod";

export const queuedResponseSchema = z.object({ "run_id": z.string() });

export type QueuedResponse = z.infer<typeof queuedResponseSchema>;

export const schema = queuedResponseSchema;
