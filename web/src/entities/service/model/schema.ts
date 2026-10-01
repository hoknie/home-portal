import type { z } from "zod";

import { diagnosisSchema, generated, serviceStateSchema, serviceStatusSchema } from "@/shared/api";
import type { Diagnosis, ServiceState, ServiceStatus } from "@/shared/api";

export { diagnosisSchema, serviceStateSchema, serviceStatusSchema };
export type { Diagnosis, ServiceState, ServiceStatus };

const served = generated.services;

export const probeKindSchema = served.probeKindSchema;

export const PROBE_KINDS = probeKindSchema.unwrap().options;

export type ProbeKind = z.infer<typeof probeKindSchema>;

export const probeSchema = served.probeSettingsSchema;

export type Probe = z.infer<typeof probeSchema>;

export const tlsModeSchema = served.tlsModeSchema;

export const TLS_MODES = tlsModeSchema.unwrap().options;

export type TlsMode = z.infer<typeof tlsModeSchema>;

export const tlsPolicySchema = served.tlsPolicySchema;

export type TlsPolicy = z.infer<typeof tlsPolicySchema>;

export const publicationSchema = served.publicationSchema;

export type Publication = z.infer<typeof publicationSchema>;

export const serviceSchema = served.serviceResponseSchema;

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

export const servicesSchema = served.servicesResponseSchema;

export type Services = z.infer<typeof servicesSchema>;
