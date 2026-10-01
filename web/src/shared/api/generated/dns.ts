import { z } from "zod";

export const transportResponseSchema = z.object({ "address": z.string().nullable(), "listening": z.boolean(), "reason": z.string().nullable() });

export type TransportResponse = z.infer<typeof transportResponseSchema>;

export const recordResponseSchema = z.object({ "type": z.string(), "value": z.string() });

export type RecordResponse = z.infer<typeof recordResponseSchema>;

export const environmentAnswerResponseSchema = z.object({ "environment": z.string(), "records": z.array(recordResponseSchema) });

export type EnvironmentAnswerResponse = z.infer<typeof environmentAnswerResponseSchema>;

export const nameResponseSchema = z.object({ "answers": z.array(environmentAnswerResponseSchema), "name": z.string() });

export type NameResponse = z.infer<typeof nameResponseSchema>;

export const dnsHttpsSettingsResponseSchema = z.object({ "enabled": z.boolean(), "host": z.string().nullable() });

export type DnsHttpsSettingsResponse = z.infer<typeof dnsHttpsSettingsResponseSchema>;

export const dnsTlsSettingsResponseSchema = z.object({ "certificate": z.string().nullable(), "enabled": z.boolean(), "key": z.string().nullable(), "port": z.number() });

export type DnsTlsSettingsResponse = z.infer<typeof dnsTlsSettingsResponseSchema>;

export const dnsSettingsResponseSchema = z.object({ "address": z.string(), "addresses": z.record(z.string(), z.array(z.string())), "https": dnsHttpsSettingsResponseSchema, "port": z.number(), "tls": dnsTlsSettingsResponseSchema, "ttl": z.number(), "zones": z.array(z.string()) });

export type DnsSettingsResponse = z.infer<typeof dnsSettingsResponseSchema>;

export const zoneResponseSchema = z.object({ "apex": z.string(), "serial": z.number(), "single": z.boolean() });

export type ZoneResponse = z.infer<typeof zoneResponseSchema>;

export const dnsResponseSchema = z.object({ "doh_url": z.string().nullable(), "enabled": z.boolean(), "environments": z.array(z.string()), "https": transportResponseSchema, "last_error": z.string().nullable(), "names": z.array(nameResponseSchema), "plain": transportResponseSchema, "settings": dnsSettingsResponseSchema, "tls": transportResponseSchema, "tls_host": z.string().nullable(), "unaddressed": z.array(z.string()), "zones": z.array(zoneResponseSchema) });

export type DnsResponse = z.infer<typeof dnsResponseSchema>;

export const schema = dnsResponseSchema;
