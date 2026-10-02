import { z } from "zod";

export const widgetActedResponseSchema = z.object({ "run_id": z.string().nullable() });

export type WidgetActedResponse = z.infer<typeof widgetActedResponseSchema>;

export const schema = widgetActedResponseSchema;
