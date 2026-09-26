import { z } from "zod";

const transportSchema = z.object({ listening: z.boolean(), address: z.string().nullable(), reason: z.string().nullable() });

export type DnsTransport = z.infer<typeof transportSchema>;

export const dnsSchema = z.object({
  enabled: z.boolean(),
  plain: transportSchema,
  tls: transportSchema,
  https: transportSchema,
  last_error: z.string().nullable(),
  zones: z.array(z.object({ apex: z.string(), single: z.boolean(), serial: z.number() })),
  names: z.array(
    z.object({
      name: z.string(),
      answers: z.array(z.object({ environment: z.string(), records: z.array(z.object({ type: z.string(), value: z.string() })) })),
    }),
  ),
  environments: z.array(z.string()),
  unaddressed: z.array(z.string()),
  tls_host: z.string().nullable(),
  doh_url: z.string().nullable(),
  settings: z.object({
    address: z.string(),
    port: z.number(),
    zones: z.array(z.string()),
    ttl: z.number(),
    addresses: z.record(z.string(), z.array(z.string())),
    tls: z.object({ enabled: z.boolean(), port: z.number(), certificate: z.string().nullable(), key: z.string().nullable() }),
    https: z.object({ enabled: z.boolean(), host: z.string().nullable() }),
  }),
});

export type Dns = z.infer<typeof dnsSchema>;

export function answerIn(dns: Dns, name: string, environment: string) {
  const found = dns.names.find((entry) => entry.name === name)?.answers.find((answer) => answer.environment === environment);
  return found?.records ?? [];
}
