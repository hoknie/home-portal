import { Activity, CalendarDays, CloudSun, LayoutGrid, type LucideIcon, Server, Sparkles } from "lucide-react";
import type { ComponentType } from "react";
import { z } from "zod";

import { calendarSchema, customWidgetSchema, metricsSchema, weatherSchema } from "@/entities/widget";
import type { ModuleName } from "@/entities/module";
import type { ServiceView } from "@/entities/service";

import { CalendarWidget } from "../ui/kinds/calendar-widget";
import { CustomWidgetKind } from "../ui/kinds/custom/custom-widget";
import { HostMetricsWidget } from "../ui/kinds/host-metrics-widget";
import { ServicesWidget } from "../ui/kinds/services-widget";
import { StatusSummaryWidget } from "../ui/kinds/status-summary-widget";
import { WeatherWidget } from "../ui/kinds/weather-widget";

export type WidgetProps<Settings, Data> = {
  settings: Settings;
  services: ServiceView[];
  data: Data;
  scope: "private" | "public";
  id: string | null;
};

export type WidgetTitleKey =
  | "widgets.statusSummary.title"
  | "widgets.services.title"
  | "widgets.metrics.title"
  | "widgets.weather.title"
  | "widgets.calendar.title"
  | "widgets.custom.title";

export type WidgetEntry = Entry<unknown, unknown>;

type Entry<Settings, Data> = {
  schema: z.ZodType<Settings>;
  data?: z.ZodType<Data>;
  component: ComponentType<WidgetProps<Settings, Data>>;
  titleKey: WidgetTitleKey;
  descriptionKey: WidgetDescriptionKey;
  icon: LucideIcon;
  module?: ModuleName;
};

export type WidgetDescriptionKey =
  | "widgets.statusSummary.description"
  | "widgets.services.description"
  | "widgets.metrics.description"
  | "widgets.weather.description"
  | "widgets.calendar.description"
  | "widgets.custom.description";

export const servicesSettingsSchema = z.object({ groups: z.array(z.string()).optional() });

export type ServicesSettings = z.infer<typeof servicesSettingsSchema>;

const anything = z.object({}).loose();

const entry = <Settings, Data>(value: Entry<Settings, Data>) => value as WidgetEntry;

export const WIDGETS: Record<string, WidgetEntry> = {
  "status-summary": entry({
    schema: anything,
    component: StatusSummaryWidget,
    titleKey: "widgets.statusSummary.title",
    descriptionKey: "widgets.statusSummary.description",
    icon: Activity,
  }),
  services: entry({
    schema: servicesSettingsSchema,
    component: ServicesWidget,
    titleKey: "widgets.services.title",
    descriptionKey: "widgets.services.description",
    icon: LayoutGrid,
  }),
  "host-metrics": entry({
    schema: anything,
    data: metricsSchema,
    component: HostMetricsWidget,
    titleKey: "widgets.metrics.title",
    descriptionKey: "widgets.metrics.description",
    icon: Server,
  }),
  weather: entry({
    schema: anything,
    data: weatherSchema,
    component: WeatherWidget,
    titleKey: "widgets.weather.title",
    descriptionKey: "widgets.weather.description",
    icon: CloudSun,
  }),
  calendar: entry({
    schema: anything,
    data: calendarSchema,
    component: CalendarWidget,
    titleKey: "widgets.calendar.title",
    descriptionKey: "widgets.calendar.description",
    icon: CalendarDays,
  }),
  custom: entry({
    schema: anything,
    data: customWidgetSchema,
    component: CustomWidgetKind,
    titleKey: "widgets.custom.title",
    descriptionKey: "widgets.custom.description",
    icon: Sparkles,
    module: "automations",
  }),
};
