import { expect, it } from "vitest";

import { settingsErrors } from "./settings-fields";

it("mirrors the weather provider: coordinates in range, a zone name and 1 to 7 days", () => {
  expect(settingsErrors("weather", { latitude: 56.95, longitude: 24.11, timezone: "Europe/Riga", days: 3 })).toEqual({});
  expect(settingsErrors("weather", { latitude: 120, longitude: 24.11, timezone: " ", days: 9 })).toEqual({
    latitude: "widgetSettings.latitudeRule",
    timezone: "widgetSettings.required",
    days: "widgetSettings.weatherDaysRule",
  });
});

it("mirrors the calendar provider: an http address, days and a limit in range", () => {
  expect(settingsErrors("calendar", { url: "https://calendar.example.com/home.ics", days: 7, limit: 10 })).toEqual({});
  expect(settingsErrors("calendar", { url: "ftp://x", limit: 99 })).toEqual({ url: "widgetSettings.urlRule", limit: "widgetSettings.limitRule" });
});

it("host metrics take absolute mount points and a type without a description has nothing to check", () => {
  expect(settingsErrors("host-metrics", { disks: ["disk"] })).toEqual({ disks: "widgetSettings.diskRule" });
  expect(settingsErrors("traffic", { anything: true })).toEqual({});
});
