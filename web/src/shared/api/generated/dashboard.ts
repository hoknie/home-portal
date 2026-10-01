import { z } from "zod";

export const sectionViewSchema = z.object({ "id": z.string(), "title": z.string().nullable() });

export type SectionView = z.infer<typeof sectionViewSchema>;

export const widgetSizeSchema = z.enum(["quarter", "third", "half", "two-thirds", "full"]).catch("full");

export type WidgetSize = z.infer<typeof widgetSizeSchema>;

export const widgetViewSchema = z.object({ "environments": z.array(z.string()).nullable(), "id": z.string().nullable(), "key": z.string(), "public": z.boolean(), "section": z.string().nullable(), "settings": z.record(z.string(), z.unknown()), "size": widgetSizeSchema, "title": z.string().nullable(), "type": z.string() });

export type WidgetView = z.infer<typeof widgetViewSchema>;

export const dashboardResponseSchema = z.object({ "sections": z.array(sectionViewSchema), "widgets": z.array(widgetViewSchema) });

export type DashboardResponse = z.infer<typeof dashboardResponseSchema>;

export const schema = dashboardResponseSchema;
