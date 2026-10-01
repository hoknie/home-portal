import { z } from "zod";

export const environmentSchema = z.string();

export type Environment = z.infer<typeof environmentSchema>;

export const publicSectionSchema = z.object({ "id": z.string(), "title": z.string().nullable() });

export type PublicSection = z.infer<typeof publicSectionSchema>;

export const diagnosisSchema = z.enum(["local-network-denied", "refused", "timeout", "host-unreachable", "name-not-resolved", "tls", "not-http", "http-status", "icmp-not-permitted", "other"]).catch("other");

export type Diagnosis = z.infer<typeof diagnosisSchema>;

export const serviceStateSchema = z.enum(["unknown", "up", "degraded", "down", "unreadable"]).catch("unknown");

export type ServiceState = z.infer<typeof serviceStateSchema>;

export const serviceStatusSchema = z.object({ "checked_at": z.string().nullable(), "diagnosis": diagnosisSchema.nullable(), "last_error": z.string().nullable(), "last_ok_at": z.string().nullable(), "latency_milliseconds": z.number().nullable(), "since": z.string(), "state": serviceStateSchema });

export type ServiceStatus = z.infer<typeof serviceStatusSchema>;

export const publicServiceSchema = z.object({ "address": z.string(), "description": z.string().nullable(), "group": z.string().nullable(), "icon": z.string().nullable(), "id": z.string(), "name": z.string(), "status": serviceStatusSchema.nullable() });

export type PublicService = z.infer<typeof publicServiceSchema>;

export const widgetSizeSchema = z.enum(["quarter", "third", "half", "two-thirds", "full"]).catch("full");

export type WidgetSize = z.infer<typeof widgetSizeSchema>;

export const publicWidgetSchema = z.object({ "id": z.string().nullable(), "section": z.string().nullable(), "settings": z.record(z.string(), z.unknown()), "size": widgetSizeSchema, "title": z.string().nullable(), "type": z.string() });

export type PublicWidget = z.infer<typeof publicWidgetSchema>;

export const portalResponseSchema = z.object({ "detected": environmentSchema, "environment": environmentSchema, "environments": z.array(environmentSchema).nullable().default(null), "sections": z.array(publicSectionSchema), "services": z.array(publicServiceSchema), "switchable": z.boolean(), "widgets": z.array(publicWidgetSchema) });

export type PortalResponse = z.infer<typeof portalResponseSchema>;

export const schema = portalResponseSchema;
