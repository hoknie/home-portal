import { z } from "zod";

export const acceptedResponseSchema = z.object({ "accepted": z.boolean(), "run_id": z.string().nullable().default(null) });

export type AcceptedResponse = z.infer<typeof acceptedResponseSchema>;

export const schema = acceptedResponseSchema;
