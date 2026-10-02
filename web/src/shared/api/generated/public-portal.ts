import { z } from "zod";

export const environmentSchema = z.string();

export type Environment = z.infer<typeof environmentSchema>;

export const sectionSurfaceSchema = z.enum(["none", "card"]).catch("none");

export type SectionSurface = z.infer<typeof sectionSurfaceSchema>;

export const titleVisibilitySchema = z.enum(["shown", "hidden"]).catch("shown");

export type TitleVisibility = z.infer<typeof titleVisibilitySchema>;

export const resolvedSectionAppearanceSchema = z.object({ "surface": sectionSurfaceSchema, "title": titleVisibilitySchema });

export type ResolvedSectionAppearance = z.infer<typeof resolvedSectionAppearanceSchema>;

export const publicSectionSchema = z.object({ "appearance": resolvedSectionAppearanceSchema, "id": z.string(), "title": z.string().nullable() });

export type PublicSection = z.infer<typeof publicSectionSchema>;

export const diagnosisSchema = z.enum(["local-network-denied", "refused", "timeout", "host-unreachable", "name-not-resolved", "tls", "not-http", "http-status", "icmp-not-permitted", "other"]).catch("other");

export type Diagnosis = z.infer<typeof diagnosisSchema>;

export const serviceStateSchema = z.enum(["unknown", "up", "degraded", "down", "unreadable"]).catch("unknown");

export type ServiceState = z.infer<typeof serviceStateSchema>;

export const serviceStatusSchema = z.object({ "checked_at": z.string().nullable(), "diagnosis": diagnosisSchema.nullable(), "last_error": z.string().nullable(), "last_ok_at": z.string().nullable(), "latency_milliseconds": z.number().nullable(), "since": z.string(), "state": serviceStateSchema });

export type ServiceStatus = z.infer<typeof serviceStatusSchema>;

export const publicServiceSchema = z.object({ "address": z.string(), "description": z.string().nullable(), "group": z.string().nullable(), "icon": z.string().nullable(), "id": z.string(), "name": z.string(), "status": serviceStatusSchema.nullable() });

export type PublicService = z.infer<typeof publicServiceSchema>;

export const accentSchema = z.enum(["neutral", "blue", "green", "amber", "red", "violet", "pink", "teal"]).catch("neutral");

export type Accent = z.infer<typeof accentSchema>;

export const alignSchema = z.enum(["start", "center", "end"]).catch("start");

export type Align = z.infer<typeof alignSchema>;

export const paddingSchema = z.enum(["normal", "compact", "none"]).catch("normal");

export type Padding = z.infer<typeof paddingSchema>;

export const surfaceSchema = z.enum(["card", "plain", "tinted", "outline"]).catch("card");

export type Surface = z.infer<typeof surfaceSchema>;

export const resolvedAppearanceSchema = z.object({ "accent": accentSchema, "align": alignSchema, "padding": paddingSchema, "surface": surfaceSchema, "title": titleVisibilitySchema });

export type ResolvedAppearance = z.infer<typeof resolvedAppearanceSchema>;

export const widgetHeightSchema = z.union([z.literal("auto"), z.number()]);

export type WidgetHeight = z.infer<typeof widgetHeightSchema>;

export const publicWidgetSchema = z.object({ "appearance": resolvedAppearanceSchema, "column": z.number().nullable(), "height": widgetHeightSchema, "id": z.string().nullable(), "row": z.number().nullable(), "section": z.string().nullable(), "settings": z.record(z.string(), z.unknown()), "title": z.string().nullable(), "type": z.string(), "width": z.number() });

export type PublicWidget = z.infer<typeof publicWidgetSchema>;

export const portalResponseSchema = z.object({ "detected": environmentSchema, "environment": environmentSchema, "environments": z.array(environmentSchema).nullable().default(null), "sections": z.array(publicSectionSchema), "services": z.array(publicServiceSchema), "switchable": z.boolean(), "widgets": z.array(publicWidgetSchema) });

export type PortalResponse = z.infer<typeof portalResponseSchema>;

export const schema = portalResponseSchema;
