import { z } from "zod";

export const accentSchema = z.enum(["neutral", "blue", "green", "amber", "red", "violet", "pink", "teal"]).catch("neutral");

export type Accent = z.infer<typeof accentSchema>;

export const alignSchema = z.enum(["start", "center", "end"]).catch("start");

export type Align = z.infer<typeof alignSchema>;

export const paddingSchema = z.enum(["normal", "compact", "none"]).catch("normal");

export type Padding = z.infer<typeof paddingSchema>;

export const surfaceSchema = z.enum(["card", "plain", "tinted", "outline"]).catch("card");

export type Surface = z.infer<typeof surfaceSchema>;

export const titleVisibilitySchema = z.enum(["shown", "hidden"]).catch("shown");

export type TitleVisibility = z.infer<typeof titleVisibilitySchema>;

export const resolvedAppearanceSchema = z.object({ "accent": accentSchema, "align": alignSchema, "padding": paddingSchema, "surface": surfaceSchema, "title": titleVisibilitySchema });

export type ResolvedAppearance = z.infer<typeof resolvedAppearanceSchema>;

export const widgetHeightSchema = z.union([z.literal("auto"), z.number()]);

export type WidgetHeight = z.infer<typeof widgetHeightSchema>;

export const libraryWidgetViewSchema = z.object({ "appearance": resolvedAppearanceSchema, "environments": z.array(z.string()).nullable(), "height": widgetHeightSchema, "id": z.string(), "placed": z.number(), "public": z.boolean(), "settings": z.record(z.string(), z.unknown()), "title": z.string().nullable(), "type": z.string(), "width": z.number() });

export type LibraryWidgetView = z.infer<typeof libraryWidgetViewSchema>;

export const libraryResponseSchema = z.object({ "widgets": z.array(libraryWidgetViewSchema) });

export type LibraryResponse = z.infer<typeof libraryResponseSchema>;

export const schema = libraryResponseSchema;
