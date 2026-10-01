import type { z } from "zod";

import { generated } from "@/shared/api";

export const widgetSchema = generated.dashboard.widgetViewSchema;

export type Widget = z.infer<typeof widgetSchema>;

export const dashboardSchema = generated.dashboard.dashboardResponseSchema;

export type Dashboard = z.infer<typeof dashboardSchema>;
