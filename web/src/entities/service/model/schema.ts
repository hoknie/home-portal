import { z } from "zod";

import { diagnosisSchema, serviceStateSchema, serviceStatusSchema } from "@/shared/api";
import type { Diagnosis, ServiceState, ServiceStatus } from "@/shared/api";

export { diagnosisSchema, serviceStateSchema, serviceStatusSchema };
export type { Diagnosis, ServiceState, ServiceStatus };

export const PROBE_KINDS = ["http", "tcp", "icmp"] as const;

export const probeKindSchema = z.enum(PROBE_KINDS).catch("http");

export type ProbeKind = z.infer<typeof probeKindSchema>;

export const probeSchema = z.object({
  enabled: z.boolean(),
  kind: probeKindSchema.default("http"),
  environment: z.string().nullable().default(null),
  path: z.string(),
  port: z.number().nullable().default(null),
  every_seconds: z.number(),
  timeout_seconds: z.number(),
  degraded_after_milliseconds: z.number(),
});

export type Probe = z.infer<typeof probeSchema>;

export const TLS_MODES = ["acme", "internal", "files"] as const;

export const tlsModeSchema = z.enum(TLS_MODES).catch("acme");

export type TlsMode = z.infer<typeof tlsModeSchema>;

export const tlsPolicySchema = z.object({
  mode: tlsModeSchema,
  email: z.string().nullable().default(null),
  certificate: z.string().nullable().default(null),
  key: z.string().nullable().default(null),
});

export type TlsPolicy = z.infer<typeof tlsPolicySchema>;

export const publicationSchema = z.object({
  host: z.string(),
  upstream: z.string().nullable().default(null),
  environments: z.array(z.string()).default(["internet"]),
  auth: z.array(z.string()).default([]),
  tls: tlsPolicySchema.nullable().default(null),
  upstream_verify: z.boolean().default(true),
});

export type Publication = z.infer<typeof publicationSchema>;

export const serviceSchema = z.object({
  id: z.string(),
  name: z.string(),
  url: z.string(),
  address: z.string().catch(""),
  probe_address: z.string().catch(""),
  addresses: z.record(z.string(), z.string()).default({}),
  environments: z.array(z.string()).nullable().default(null),
  public: z.boolean().default(false),
  public_status: z.boolean().default(false),
  notify: z.boolean().default(true),
  links: z.array(z.object({ title: z.string(), url: z.string() })).default([]),
  notes: z.string().nullable().default(null),
  widgets: z.array(z.string()).default([]),
  group: z.string().nullable(),
  icon: z.string().nullable(),
  description: z.string().nullable(),
  probe: probeSchema,
  proxy: publicationSchema.nullable().default(null),
  status: serviceStatusSchema,
});

export type Service = z.infer<typeof serviceSchema>;

export type ServiceView = {
  id: string;
  name: string;
  url?: string;
  address: string;
  group: string | null;
  icon: string | null;
  description: string | null;
  status: ServiceStatus | null;
};

export const servicesSchema = z.object({ services: z.array(serviceSchema) });

export type Services = z.infer<typeof servicesSchema>;
