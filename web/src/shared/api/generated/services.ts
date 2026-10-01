import { z } from "zod";

export const serviceLinkSchema = z.object({ "title": z.string(), "url": z.string() });

export type ServiceLink = z.infer<typeof serviceLinkSchema>;

export const probeKindSchema = z.enum(["http", "tcp", "icmp"]).catch("http");

export type ProbeKind = z.infer<typeof probeKindSchema>;

export const probeSettingsSchema = z.object({ "degraded_after_milliseconds": z.number(), "enabled": z.boolean(), "environment": z.string().nullable(), "every_seconds": z.number(), "kind": probeKindSchema, "path": z.string(), "port": z.number().nullable(), "timeout_seconds": z.number() });

export type ProbeSettings = z.infer<typeof probeSettingsSchema>;

export const tlsModeSchema = z.enum(["acme", "internal", "files"]).catch("acme");

export type TlsMode = z.infer<typeof tlsModeSchema>;

export const tlsPolicySchema = z.object({ "certificate": z.string().nullable(), "email": z.string().nullable(), "key": z.string().nullable(), "mode": tlsModeSchema });

export type TlsPolicy = z.infer<typeof tlsPolicySchema>;

export const publicationSchema = z.object({ "auth": z.array(z.string()), "environments": z.array(z.string()), "host": z.string(), "tls": tlsPolicySchema.nullable(), "upstream": z.string().nullable(), "upstream_verify": z.boolean() });

export type Publication = z.infer<typeof publicationSchema>;

export const diagnosisSchema = z.enum(["local-network-denied", "refused", "timeout", "host-unreachable", "name-not-resolved", "tls", "not-http", "http-status", "icmp-not-permitted", "other"]).catch("other");

export type Diagnosis = z.infer<typeof diagnosisSchema>;

export const serviceStateSchema = z.enum(["unknown", "up", "degraded", "down", "unreadable"]).catch("unknown");

export type ServiceState = z.infer<typeof serviceStateSchema>;

export const serviceStatusSchema = z.object({ "checked_at": z.string().nullable(), "diagnosis": diagnosisSchema.nullable(), "last_error": z.string().nullable(), "last_ok_at": z.string().nullable(), "latency_milliseconds": z.number().nullable(), "since": z.string(), "state": serviceStateSchema });

export type ServiceStatus = z.infer<typeof serviceStatusSchema>;

export const serviceResponseSchema = z.object({ "address": z.string(), "addresses": z.record(z.string(), z.string()), "description": z.string().nullable(), "environments": z.array(z.string()).nullable(), "group": z.string().nullable(), "icon": z.string().nullable(), "id": z.string(), "links": z.array(serviceLinkSchema), "name": z.string(), "notes": z.string().nullable(), "notify": z.boolean(), "probe": probeSettingsSchema, "probe_address": z.string(), "proxy": publicationSchema.nullable(), "public": z.boolean(), "public_status": z.boolean(), "status": serviceStatusSchema, "url": z.string(), "widgets": z.array(z.string()) });

export type ServiceResponse = z.infer<typeof serviceResponseSchema>;

export const servicesResponseSchema = z.object({ "services": z.array(serviceResponseSchema) });

export type ServicesResponse = z.infer<typeof servicesResponseSchema>;

export const schema = servicesResponseSchema;
