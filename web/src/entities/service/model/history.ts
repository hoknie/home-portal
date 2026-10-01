import { z } from "zod";

import { generated } from "@/shared/api";

export const HISTORY_RANGES = ["1h", "6h", "24h", "7d", "30d"] as const;

export const historyRangeSchema = z.enum(HISTORY_RANGES);

export type HistoryRange = z.infer<typeof historyRangeSchema>;

const served = generated.history;

export const uptimeSchema = served.uptimeResponseSchema;

export type Uptime = z.infer<typeof uptimeSchema>;

export const latencyPointSchema = served.latencyPointResponseSchema;

export type LatencyPoint = z.infer<typeof latencyPointSchema>;

export const transitionSchema = served.transitionResponseSchema;

export type Transition = z.infer<typeof transitionSchema>;

export const historySchema = served.historyResponseSchema;

export type History = z.infer<typeof historySchema>;
