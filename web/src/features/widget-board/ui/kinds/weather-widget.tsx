"use client";

import { CloudRain, CloudSnow, Cloudy, Haze, Sun, Zap } from "lucide-react";
import { useFormatter, useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { Weather } from "@/entities/widget";
import { StatTile } from "@/shared/ui/stat-tile";

export type WeatherProps = { data: Weather };

const SKY = "size-10 shrink-0 text-primary";

const ICONS: Record<string, ReactNode> = {
  clear: <Sun className={SKY} aria-hidden />,
  "mostly-clear": <Sun className={SKY} aria-hidden />,
  "partly-cloudy": <Cloudy className={SKY} aria-hidden />,
  overcast: <Cloudy className={SKY} aria-hidden />,
  fog: <Haze className={SKY} aria-hidden />,
  "rime-fog": <Haze className={SKY} aria-hidden />,
  drizzle: <CloudRain className={SKY} aria-hidden />,
  "freezing-drizzle": <CloudRain className={SKY} aria-hidden />,
  rain: <CloudRain className={SKY} aria-hidden />,
  "heavy-rain": <CloudRain className={SKY} aria-hidden />,
  "freezing-rain": <CloudRain className={SKY} aria-hidden />,
  showers: <CloudRain className={SKY} aria-hidden />,
  "heavy-showers": <CloudRain className={SKY} aria-hidden />,
  snow: <CloudSnow className={SKY} aria-hidden />,
  "heavy-snow": <CloudSnow className={SKY} aria-hidden />,
  "snow-grains": <CloudSnow className={SKY} aria-hidden />,
  "snow-showers": <CloudSnow className={SKY} aria-hidden />,
  thunderstorm: <Zap className={SKY} aria-hidden />,
  "thunderstorm-hail": <Zap className={SKY} aria-hidden />,
};

const UNKNOWN_SKY = <Cloudy className={SKY} aria-hidden />;

export function iconOf(condition: string): ReactNode {
  return ICONS[condition] ?? UNKNOWN_SKY;
}

export function WeatherWidget({ data }: WeatherProps) {
  const t = useTranslations("widgets.weather");
  const conditions = useTranslations("conditions");
  const format = useFormatter();
  const degree = data.units === "metric" ? "°C" : "°F";
  const name = (condition: string) =>
    conditions.has(condition as Parameters<typeof conditions.has>[0])
      ? conditions(condition as Parameters<typeof conditions>[0])
      : condition;
  return (
    <div className="grid gap-4 rounded-xl border bg-card p-4">
      <div className="flex items-center gap-4">
        {iconOf(data.current.condition)}
        <div className="min-w-0">
          <p className="text-3xl font-semibold tabular-nums">
            {Math.round(data.current.temperature)}
            {degree}
          </p>
          <p className="truncate text-sm text-muted-foreground">{name(data.current.condition)}</p>
        </div>
        <div className="ml-auto grid gap-1 text-right text-xs text-muted-foreground">
          <span>{t("feelsLike", { value: Math.round(data.current.apparent_temperature), degree })}</span>
          <span>{t("wind", { value: Math.round(data.current.wind_speed) })}</span>
          {data.current.humidity_percent === null ? null : <span>{t("humidity", { value: data.current.humidity_percent })}</span>}
        </div>
      </div>
      <div className="grid grid-cols-2 gap-3 @sm:grid-cols-3">
        {data.daily.map((day) => (
          <StatTile
            key={day.date}
            label={format.dateTime(new Date(day.date), { weekday: "short", day: "numeric", month: "short" })}
            value={`${Math.round(day.temperature_maximum)}${degree}`}
            hint={t("night", { value: Math.round(day.temperature_minimum), degree })}
          />
        ))}
      </div>
    </div>
  );
}
