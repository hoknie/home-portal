import { z } from "zod";

import { sectionSchema, widgetSizeSchema } from "@/shared/api";

export const widgetSchema = z.object({
  key: z.string().catch(""),
  type: z.string(),
  id: z.string().nullable().catch(null),
  title: z.string().nullable(),
  settings: z.record(z.string(), z.unknown()),
  section: z.string().nullable().catch(null),
  size: widgetSizeSchema.default("full"),
  environments: z.array(z.string()).nullable().catch(null),
  public: z.boolean().catch(false),
});

export type Widget = z.infer<typeof widgetSchema>;

export const dashboardSchema = z.object({
  sections: z.array(sectionSchema).default([{ id: "main", title: null }]),
  widgets: z.array(widgetSchema),
});

export type Dashboard = z.infer<typeof dashboardSchema>;
