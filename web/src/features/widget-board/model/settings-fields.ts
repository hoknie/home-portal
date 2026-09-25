import { z } from "zod";

export type SettingsFieldKind = "text" | "number" | "list" | "select" | "groups";

export type SettingsLabel =
  | "widgetSettings.groups"
  | "widgetSettings.disks"
  | "widgetSettings.latitude"
  | "widgetSettings.longitude"
  | "widgetSettings.timezone"
  | "widgetSettings.zone"
  | "widgetSettings.units"
  | "widgetSettings.days"
  | "widgetSettings.url"
  | "widgetSettings.limit"
  | "widgetSettings.secret";

export type SettingsField = {
  key: string;
  label: SettingsLabel;
  kind: SettingsFieldKind;
  options?: readonly string[];
  optional?: boolean;
};

export type SettingsDescription = { fields: SettingsField[]; schema: z.ZodType };

const offset = z.string().regex(/^[+-]\d{2}:\d{2}$/, "widgetSettings.timezoneRule");

export const SETTINGS: Record<string, SettingsDescription> = {
  services: {
    fields: [{ key: "groups", label: "widgetSettings.groups", kind: "groups", optional: true }],
    schema: z.object({ groups: z.array(z.string().min(1)).optional() }),
  },
  "host-metrics": {
    fields: [{ key: "disks", label: "widgetSettings.disks", kind: "list", optional: true }],
    schema: z.object({ disks: z.array(z.string().startsWith("/", "widgetSettings.diskRule")).optional() }),
  },
  weather: {
    fields: [
      { key: "latitude", label: "widgetSettings.latitude", kind: "number" },
      { key: "longitude", label: "widgetSettings.longitude", kind: "number" },
      { key: "timezone", label: "widgetSettings.zone", kind: "text", optional: true },
      { key: "units", label: "widgetSettings.units", kind: "select", options: ["metric", "imperial"], optional: true },
      { key: "days", label: "widgetSettings.days", kind: "number", optional: true },
    ],
    schema: z.object({
      latitude: z.number("widgetSettings.required").min(-90, "widgetSettings.latitudeRule").max(90, "widgetSettings.latitudeRule"),
      longitude: z.number("widgetSettings.required").min(-180, "widgetSettings.longitudeRule").max(180, "widgetSettings.longitudeRule"),
      timezone: z.string().trim().min(1, "widgetSettings.required").optional(),
      units: z.enum(["metric", "imperial"]).optional(),
      days: z.number().int().min(1, "widgetSettings.weatherDaysRule").max(7, "widgetSettings.weatherDaysRule").optional(),
    }),
  },
  calendar: {
    fields: [
      { key: "url", label: "widgetSettings.url", kind: "text" },
      { key: "days", label: "widgetSettings.days", kind: "number", optional: true },
      { key: "limit", label: "widgetSettings.limit", kind: "number", optional: true },
      { key: "timezone", label: "widgetSettings.timezone", kind: "text", optional: true },
      { key: "secret", label: "widgetSettings.secret", kind: "text", optional: true },
    ],
    schema: z.object({
      url: z.url({ protocol: /^https?$/, error: "widgetSettings.urlRule" }),
      days: z.number().int().min(1, "widgetSettings.calendarDaysRule").max(31, "widgetSettings.calendarDaysRule").optional(),
      limit: z.number().int().min(1, "widgetSettings.limitRule").max(50, "widgetSettings.limitRule").optional(),
      timezone: offset.optional(),
      secret: z.string().min(1).optional(),
    }),
  },
};

export function settingsErrors(type: string, value: Record<string, unknown>): Record<string, string> {
  const description = SETTINGS[type];
  if (!description) {
    return {};
  }
  const parsed = description.schema.safeParse(value);
  if (parsed.success) {
    return {};
  }
  return Object.fromEntries(parsed.error.issues.map((issue) => [String(issue.path[0] ?? ""), issue.message]));
}
