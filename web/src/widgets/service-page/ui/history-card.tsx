"use client";

import { useFormatter, useNow, useTranslations } from "next-intl";
import { useState } from "react";

import { HISTORY_RANGES, type HistoryRange, useServiceHistory } from "@/entities/service";
import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";
import { HistoryChart } from "@/shared/ui/history-chart";
import { Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";
import { StatTile } from "@/shared/ui/stat-tile";

import { durationParts, percent } from "../model/history";
import { TransitionsList } from "./transitions-list";

export function HistoryCard({ id }: { id: string }) {
  const t = useTranslations("servicePage");
  const root = useTranslations();
  const format = useFormatter();
  const now = useNow({ updateInterval: STATUS_REFRESH_MILLISECONDS });
  const [range, setRange] = useState<HistoryRange>("24h");
  const history = useServiceHistory(id, range);
  const rangeName = (value: HistoryRange) => t(`history.ranges.${value}`);
  const moment = (at: string | number) => format.dateTime(new Date(at), { dateStyle: "short", timeStyle: "short" });
  const switcher = (
    <div className="flex gap-1" role="group" aria-label={t("history.title")}>
      {HISTORY_RANGES.map((value) => (
        <Button key={value} size="sm" variant={value === range ? "secondary" : "ghost"} aria-pressed={value === range} onClick={() => setRange(value)}>
          {rangeName(value)}
        </Button>
      ))}
    </div>
  );
  if (!history.data) {
    return (
      <SectionCard title={t("history.title")} actions={switcher}>
        <Skeleton className="h-48 w-full" aria-busy="true" />
      </SectionCard>
    );
  }
  const data = history.data;
  const to = Date.parse(data.to);
  const from = Date.parse(data.from);
  const milliseconds = (value: number) => root("common.milliseconds", { value: Math.round(value) });
  const points = data.points.map((point) => ({ at: Date.parse(point.at), average: point.average, minimum: point.minimum, maximum: point.maximum, state: point.state === "unknown" ? null : point.state }));
  const highest = Math.max(0, ...points.map((point) => point.maximum ?? point.average ?? 0));
  const failed = points.filter((point) => point.state === "down" || point.state === "unreadable").length;
  const summary = t("history.summary", { range: rangeName(range), highest: milliseconds(highest), failed, checks: points.length });
  const tick = (at: number) =>
    range === "1h" || range === "6h" || range === "24h" ? format.dateTime(new Date(at), { timeStyle: "short" }) : format.dateTime(new Date(at), { day: "numeric", month: "short" });
  return (
    <SectionCard title={t("history.title")} actions={switcher}>
      <div className="grid gap-6">
        <div className="grid gap-3 sm:grid-cols-3">
          {data.uptime.map((uptime) => {
            const value = percent(uptime.ratio);
            const known = HISTORY_RANGES.find((name) => name === uptime.range) ?? "24h";
            return (
              <StatTile
                key={uptime.range}
                label={t("history.uptime", { range: rangeName(known) })}
                value={value === null ? t("history.noData") : format.number(value / 100, { style: "percent", maximumFractionDigits: 1 })}
                hint={t("history.covered", { duration: t("duration", durationParts(uptime.covered_seconds * 1000)) })}
              />
            );
          })}
        </div>
        <HistoryChart
          from={from}
          to={to}
          step={data.step_seconds * 1000}
          formatTime={tick}
          formatValue={milliseconds}
          title={t("history.chart", { range: rangeName(range) })}
          summary={summary}
          empty={t("history.noSamples")}
          legend={{ average: t("history.average"), range: t("history.range"), states: { unknown: root("status.unknown"), up: root("status.up"), degraded: root("status.degraded"), down: root("status.down"), unreadable: root("status.unreadable") } }}
          points={points}
          hintOf={(point) => ({
            time: t("history.interval", { from: moment(point.at), to: format.dateTime(new Date(point.until), { timeStyle: "short" }) }),
            state: point.state ? root(`status.${point.state}`) : t("history.noChecks"),
            average: point.average === null ? t("history.noLatency") : t("history.averageValue", { value: milliseconds(point.average) }),
            maximum: point.maximum === null || point.minimum === null ? "" : t("history.spreadValue", { minimum: milliseconds(point.minimum), maximum: milliseconds(point.maximum) }),
          })}
        />
        <TransitionsList transitions={data.transitions} now={now.getTime()} />
      </div>
    </SectionCard>
  );
}
