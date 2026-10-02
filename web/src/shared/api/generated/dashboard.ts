import { z } from "zod";

export const sectionSurfaceSchema = z.enum(["none", "card"]).catch("none");

export type SectionSurface = z.infer<typeof sectionSurfaceSchema>;

export const titleVisibilitySchema = z.enum(["shown", "hidden"]).catch("shown");

export type TitleVisibility = z.infer<typeof titleVisibilitySchema>;

export const resolvedSectionAppearanceSchema = z.object({ "surface": sectionSurfaceSchema, "title": titleVisibilitySchema });

export type ResolvedSectionAppearance = z.infer<typeof resolvedSectionAppearanceSchema>;

export const sectionViewSchema = z.object({ "appearance": resolvedSectionAppearanceSchema, "id": z.string(), "title": z.string().nullable() });

export type SectionView = z.infer<typeof sectionViewSchema>;

export const accentSchema = z.enum(["neutral", "blue", "cyan", "teal", "green", "lime", "amber", "orange", "red", "pink", "violet", "indigo"]).catch("neutral");

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

export const widgetViewSchema = z.object({ "appearance": resolvedAppearanceSchema, "column": z.number().nullable(), "environments": z.array(z.string()).nullable(), "height": widgetHeightSchema, "id": z.string().nullable(), "key": z.string(), "public": z.boolean(), "row": z.number().nullable(), "section": z.string().nullable(), "settings": z.record(z.string(), z.unknown()), "title": z.string().nullable(), "type": z.string(), "width": z.number() });

export type WidgetView = z.infer<typeof widgetViewSchema>;

export const dashboardResponseSchema = z.object({ "sections": z.array(sectionViewSchema), "widgets": z.array(widgetViewSchema) });

export type DashboardResponse = z.infer<typeof dashboardResponseSchema>;

export const schema = dashboardResponseSchema;
