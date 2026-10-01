import { z } from "zod";

export const scheduleResponseSchema = z.object({ "times": z.array(z.string()), "timezone": z.string() });

export type ScheduleResponse = z.infer<typeof scheduleResponseSchema>;

export const schema = scheduleResponseSchema;
