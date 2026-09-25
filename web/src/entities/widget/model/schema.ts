import { z } from "zod";

export const widgetDataSchema = z.object({
  data: z.unknown(),
  fetched_at: z.string(),
  stale: z.boolean(),
  problem: z.string().nullable(),
  refresh_seconds: z.number(),
});

export type WidgetData = z.infer<typeof widgetDataSchema>;

export const metricsSchema = z.object({
  hostname: z.string().nullable(),
  cpu_percent: z.number(),
  load_average: z.tuple([z.number(), z.number(), z.number()]).nullable(),
  memory: z.object({ used_bytes: z.number(), total_bytes: z.number() }),
  swap: z.object({ used_bytes: z.number(), total_bytes: z.number() }),
  disks: z.array(
    z.object({
      mount_point: z.string(),
      file_system: z.string(),
      total_bytes: z.number(),
      available_bytes: z.number(),
    }),
  ),
  uptime_seconds: z.number(),
});

export type Metrics = z.infer<typeof metricsSchema>;

export const weatherSchema = z.object({
  current: z.object({
    temperature: z.number(),
    apparent_temperature: z.number(),
    humidity_percent: z.number().nullable(),
    wind_speed: z.number(),
    condition: z.string(),
    weather_code: z.number(),
    is_day: z.boolean(),
  }),
  daily: z.array(
    z.object({
      date: z.string(),
      condition: z.string(),
      weather_code: z.number(),
      temperature_minimum: z.number(),
      temperature_maximum: z.number(),
      precipitation_chance: z.number().nullable(),
    }),
  ),
  units: z.enum(["metric", "imperial"]).catch("metric"),
  timezone: z.string(),
});

export type Weather = z.infer<typeof weatherSchema>;

export const calendarSchema = z.object({
  events: z.array(
    z.object({
      summary: z.string(),
      start: z.string(),
      end: z.string().nullable(),
      all_day: z.boolean(),
      location: z.string().nullable(),
      calendar: z.string().nullable(),
      repeats: z.enum(["never", "expanded", "unsupported"]).catch("never"),
    }),
  ),
});

export type Calendar = z.infer<typeof calendarSchema>;
