import { z } from "zod";

export const serviceStateSchema = z.enum(["unknown", "up", "degraded", "down", "unreadable"]).catch("unknown");

export type ServiceState = z.infer<typeof serviceStateSchema>;

export const latencyPointResponseSchema = z.object({ "at": z.string(), "average": z.number().nullable(), "maximum": z.number().nullable(), "minimum": z.number().nullable(), "state": serviceStateSchema });

export type LatencyPointResponse = z.infer<typeof latencyPointResponseSchema>;

export const transitionResponseSchema = z.object({ "at": z.string(), "error": z.string().nullable(), "from": serviceStateSchema, "to": serviceStateSchema });

export type TransitionResponse = z.infer<typeof transitionResponseSchema>;

export const uptimeResponseSchema = z.object({ "covered_seconds": z.number(), "range": z.string(), "ratio": z.number().nullable() });

export type UptimeResponse = z.infer<typeof uptimeResponseSchema>;

export const historyResponseSchema = z.object({ "from": z.string(), "points": z.array(latencyPointResponseSchema), "range": z.string(), "step_seconds": z.number(), "to": z.string(), "transitions": z.array(transitionResponseSchema), "uptime": z.array(uptimeResponseSchema) });

export type HistoryResponse = z.infer<typeof historyResponseSchema>;

export const schema = historyResponseSchema;
