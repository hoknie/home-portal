import type { ComponentType } from "react";
import { z } from "zod";

import { calendarSchema, metricsSchema, weatherSchema } from "@/entities/widget";
import type { ServiceView } from "@/entities/service";

import { CalendarWidget } from "../ui/kinds/calendar-widget";
import { HostMetricsWidget } from "../ui/kinds/host-metrics-widget";
import { ServicesWidget } from "../ui/kinds/services-widget";
import { StatusSummaryWidget } from "../ui/kinds/status-summary-widget";
import { WeatherWidget } from "../ui/kinds/weather-widget";

export type WidgetProps<Settings, Data> = {
  settings: Settings;
  services: ServiceView[];
  data: Data;
  scope: "private" | "public";
};

export type WidgetTitleKey =
  | "widgets.statusSummary.title"
  | "widgets.services.title"
  | "widgets.metrics.title"
  | "widgets.weather.title"
  | "widgets.calendar.title";

export type WidgetEntry = Entry<unknown, unknown>;

type Entry<Settings, Data> = {
  schema: z.ZodType<Settings>;
  data?: z.ZodType<Data>;
  component: ComponentType<WidgetProps<Settings, Data>>;
  titleKey: WidgetTitleKey;
};

export const servicesSettingsSchema = z.object({ groups: z.array(z.string()).optional() });

export type ServicesSettings = z.infer<typeof servicesSettingsSchema>;

const anything = z.object({}).loose();

const entry = <Settings, Data>(value: Entry<Settings, Data>) => value as WidgetEntry;

export const WIDGETS: Record<string, WidgetEntry> = {
  "status-summary": entry({ schema: anything, component: StatusSummaryWidget, titleKey: "widgets.statusSummary.title" }),
  services: entry({ schema: servicesSettingsSchema, component: ServicesWidget, titleKey: "widgets.services.title" }),
  "host-metrics": entry({ schema: anything, data: metricsSchema, component: HostMetricsWidget, titleKey: "widgets.metrics.title" }),
  weather: entry({ schema: anything, data: weatherSchema, component: WeatherWidget, titleKey: "widgets.weather.title" }),
  calendar: entry({ schema: anything, data: calendarSchema, component: CalendarWidget, titleKey: "widgets.calendar.title" }),
};
