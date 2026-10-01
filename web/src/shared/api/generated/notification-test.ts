import { z } from "zod";

export const deliveryResponseSchema = z.object({ "at": z.string(), "channel": z.string(), "delivered": z.boolean(), "error": z.string().nullable() });

export type DeliveryResponse = z.infer<typeof deliveryResponseSchema>;

export const schema = deliveryResponseSchema;
