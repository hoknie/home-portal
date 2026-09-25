export const CRON_PRESETS = ["hourly", "daily", "weekly", "monthly"] as const;

export type CronPreset = (typeof CRON_PRESETS)[number];

export const WEEKDAYS = [1, 2, 3, 4, 5, 6, 0] as const;

export function presetCron(preset: CronPreset, time: string, weekday: number) {
  const [hour, minute] = time.split(":").map((part) => Number.parseInt(part, 10));
  const at = `${Number.isFinite(minute) ? minute : 0} ${Number.isFinite(hour) ? hour : 0}`;
  switch (preset) {
    case "hourly":
      return "0 * * * *";
    case "daily":
      return `${at} * * *`;
    case "weekly":
      return `${at} * * ${weekday}`;
    case "monthly":
      return `${at} 1 * *`;
  }
}
