import { z } from "zod";

import { sectionSchema, serviceStatusSchema, widgetSizeSchema } from "@/shared/api";

export const publicServiceSchema = z.object({
  id: z.string(),
  name: z.string(),
  address: z.string(),
  group: z.string().nullable(),
  icon: z.string().nullable(),
  description: z.string().nullable(),
  status: serviceStatusSchema.nullable(),
});

export type PublicService = z.infer<typeof publicServiceSchema>;

export const publicWidgetSchema = z.object({
  type: z.string(),
  id: z.string().nullable(),
  title: z.string().nullable(),
  settings: z.record(z.string(), z.unknown()),
  section: z.string().nullable().catch(null),
  size: widgetSizeSchema.default("full"),
});

export type PublicWidget = z.infer<typeof publicWidgetSchema>;

export const portalSchema = z.object({
  environment: z.string(),
  detected: z.string(),
  switchable: z.boolean(),
  environments: z.array(z.string()).default([]),
  sections: z.array(sectionSchema).default([]),
  services: z.array(publicServiceSchema),
  widgets: z.array(publicWidgetSchema),
});

export type Portal = z.infer<typeof portalSchema>;
