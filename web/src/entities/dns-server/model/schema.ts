import type { z } from "zod";

import { generated } from "@/shared/api";

export type DnsTransport = z.infer<typeof generated.dns.transportResponseSchema>;

export const dnsSchema = generated.dns.dnsResponseSchema;

export type Dns = z.infer<typeof dnsSchema>;

export function answerIn(dns: Dns, name: string, environment: string) {
  const found = dns.names.find((entry) => entry.name === name)?.answers.find((answer) => answer.environment === environment);
  return found?.records ?? [];
}
