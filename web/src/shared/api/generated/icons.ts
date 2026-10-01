import { z } from "zod";

export const iconStateSchema = z.object({ "available": z.boolean(), "fetched_at": z.string().nullable(), "problem": z.string().nullable(), "service": z.string(), "source": z.string() });

export type IconState = z.infer<typeof iconStateSchema>;

export const arrayOfIconStateSchema = z.array(iconStateSchema);

export type ArrayOfIconState = z.infer<typeof arrayOfIconStateSchema>;

export const schema = arrayOfIconStateSchema;
