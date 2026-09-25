import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { calendarSchema, metricsSchema, weatherSchema, widgetDataSchema } from "./schema";

it("the metrics sample parses through the envelope and the payload", () => {
  const envelope = widgetDataSchema.parse(apiSamples.widgetHostMetrics);
  expect(envelope.stale).toBe(false);
  expect(envelope.refresh_seconds).toBe(10);
  const metrics = metricsSchema.parse(envelope.data);
  expect(metrics.disks.map((disk) => disk.mount_point)).toEqual(["/", "/media"]);
  expect(metrics.memory.total_bytes).toBeGreaterThan(metrics.memory.used_bytes);
});

it("the weather sample parses with its daily list", () => {
  const weather = weatherSchema.parse(widgetDataSchema.parse(apiSamples.widgetWeather).data);
  expect(weather.current.condition).toBe("partly-cloudy");
  expect(weather.daily).toHaveLength(2);
  expect(weather.units).toBe("metric");
});

it("the calendar sample parses and keeps an unsupported repeat", () => {
  const calendar = calendarSchema.parse(widgetDataSchema.parse(apiSamples.widgetCalendar).data);
  expect(calendar.events.map((event) => event.repeats)).toEqual(["never", "unsupported"]);
});

it("an unknown unit falls back to metric instead of failing the widget", () => {
  const sample = apiSamples.widgetWeather.data;
  expect(weatherSchema.parse({ ...sample, units: "kelvin" }).units).toBe("metric");
});

it("stale data still parses, with the problem that made it stale", () => {
  const envelope = widgetDataSchema.parse({
    ...apiSamples.widgetWeather,
    stale: true,
    problem: "open-meteo answered 503",
  });
  expect(envelope.stale).toBe(true);
  expect(envelope.problem).toBe("open-meteo answered 503");
});
