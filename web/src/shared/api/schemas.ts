import { z } from "zod";

export const fieldErrorSchema = z.object({ field: z.string(), message: z.string() });

export const fieldErrorsSchema = z.object({ errors: z.array(fieldErrorSchema) });

export const emptySchema = z.unknown();

export const serviceStateSchema = z.enum(["unknown", "up", "degraded", "down", "unreadable"]).catch("unknown");

export type ServiceState = z.infer<typeof serviceStateSchema>;

export const DIAGNOSES = [
  "local-network-denied",
  "refused",
  "timeout",
  "host-unreachable",
  "name-not-resolved",
  "tls",
  "not-http",
  "http-status",
  "icmp-not-permitted",
  "other",
] as const;

export const diagnosisSchema = z.enum(DIAGNOSES).catch("other");

export type Diagnosis = z.infer<typeof diagnosisSchema>;

export const serviceStatusSchema = z.object({
  state: serviceStateSchema,
  checked_at: z.string().nullable(),
  latency_milliseconds: z.number().nullable(),
  last_ok_at: z.string().nullable(),
  last_error: z.string().nullable(),
  diagnosis: diagnosisSchema.nullable().default(null),
  since: z.string(),
});

export type ServiceStatus = z.infer<typeof serviceStatusSchema>;

export const WIDGET_SIZES = ["quarter", "third", "half", "two-thirds", "full"] as const;

export const widgetSizeSchema = z.enum(WIDGET_SIZES).catch("full");

export type WidgetSize = z.infer<typeof widgetSizeSchema>;

export const sectionSchema = z.object({ id: z.string(), title: z.string().nullable() });

export type Section = z.infer<typeof sectionSchema>;
