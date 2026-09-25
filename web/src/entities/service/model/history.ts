import { z } from "zod";

import { serviceStateSchema } from "@/shared/api";

export const HISTORY_RANGES = ["24h", "7d", "30d"] as const;

export const historyRangeSchema = z.enum(HISTORY_RANGES);

export type HistoryRange = z.infer<typeof historyRangeSchema>;

export const uptimeSchema = z.object({
  range: z.string(),
  ratio: z.number().nullable(),
  covered_seconds: z.number(),
});

export type Uptime = z.infer<typeof uptimeSchema>;

export const latencyPointSchema = z.object({
  at: z.string(),
  state: serviceStateSchema,
  average: z.number().nullable(),
  minimum: z.number().nullable(),
  maximum: z.number().nullable(),
});

export type LatencyPoint = z.infer<typeof latencyPointSchema>;

export const transitionSchema = z.object({
  at: z.string(),
  from: serviceStateSchema,
  to: serviceStateSchema,
  error: z.string().nullable(),
});

export type Transition = z.infer<typeof transitionSchema>;

export const historySchema = z.object({
  range: z.string(),
  from: z.string(),
  to: z.string(),
  uptime: z.array(uptimeSchema),
  points: z.array(latencyPointSchema),
  transitions: z.array(transitionSchema),
});

export type History = z.infer<typeof historySchema>;
