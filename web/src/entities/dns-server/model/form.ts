import { z } from "zod";

import type { Dns } from "./schema";

export const DNS_PORT_MIN = 1;
export const DNS_PORT_MAX = 65_535;
export const DNS_TTL_MIN = 5;
export const DNS_TTL_MAX = 86_400;

const port = z.number().int().min(DNS_PORT_MIN).max(DNS_PORT_MAX);

export const dnsFormSchema = z.object({
  enabled: z.boolean(),
  address: z.string().trim().min(1),
  port,
  zones: z.array(z.string()),
  ttl: z.number().int().min(DNS_TTL_MIN).max(DNS_TTL_MAX),
  addresses: z.record(z.string(), z.string()),
  tls_enabled: z.boolean(),
  tls_port: port,
  https_enabled: z.boolean(),
  https_host: z.string(),
});

export type DnsForm = z.infer<typeof dnsFormSchema>;

export function dnsFormOf(dns: Dns): DnsForm {
  const settings = dns.settings;
  return {
    enabled: dns.enabled,
    address: settings.address,
    port: settings.port,
    zones: settings.zones,
    ttl: settings.ttl,
    addresses: Object.fromEntries(dns.environments.map((environment) => [environment, (settings.addresses[environment] ?? []).join(", ")])),
    tls_enabled: settings.tls.enabled,
    tls_port: settings.tls.port,
    https_enabled: settings.https.enabled,
    https_host: settings.https.host ?? "",
  };
}

export function dnsRequestOf(form: DnsForm, dns: Dns) {
  const addresses = Object.fromEntries(
    Object.entries(form.addresses)
      .map(([environment, text]) => [environment, text.split(",").map((part) => part.trim()).filter((part) => part !== "")] as const)
      .filter(([, found]) => found.length > 0),
  );
  return {
    enabled: form.enabled,
    address: form.address.trim(),
    port: form.port,
    zones: form.zones,
    ttl: form.ttl,
    addresses,
    tls: { enabled: form.tls_enabled, port: form.tls_port, certificate: dns.settings.tls.certificate, key: dns.settings.tls.key },
    https: { enabled: form.https_enabled, host: form.https_host.trim() === "" ? null : form.https_host.trim() },
  };
}
