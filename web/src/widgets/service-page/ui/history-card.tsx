"use client";

import { useFormatter, useNow, useTranslations } from "next-intl";
import { useState } from "react";

import { HISTORY_RANGES, type HistoryRange, useServiceHistory } from "@/entities/service";
import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";
import { LatencyChart } from "@/shared/ui/latency-chart";
import { Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";
import { StatTile } from "@/shared/ui/stat-tile";
import { UptimeStrip } from "@/shared/ui/uptime-strip";

import { durationParts, percent, slotsOf } from "../model/history";
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
  const slots = slotsOf(data.points, range, to);
  const tick = (at: number) =>
    range === "24h" ? format.dateTime(new Date(at), { timeStyle: "short" }) : format.dateTime(new Date(at), { day: "numeric", month: "short" });
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
        <div className="grid gap-2">
          <p className="text-sm text-muted-foreground">{t("history.latency", { range: rangeName(range) })}</p>
          <LatencyChart
            from={from}
            to={to}
            formatTime={tick}
            formatValue={(value) => root("common.milliseconds", { value })}
            title={t("history.latency", { range: rangeName(range) })}
            empty={t("history.noSamples")}
            samples={data.points.map((point) => ({
              at: point.at,
              value: point.average,
              failed: point.state === "down" || point.state === "unreadable",
              label:
                point.average === null
                  ? t("history.failedPoint", { moment: moment(point.at) })
                  : t("history.point", { moment: moment(point.at), value: root("common.milliseconds", { value: point.average }) }),
            }))}
          />
        </div>
        <div className="grid gap-2">
          <p className="text-sm text-muted-foreground">{t("history.strip", { range: rangeName(range) })}</p>
          <UptimeStrip
            from={from}
            to={to}
            formatTime={tick}
            title={t("history.strip", { range: rangeName(range) })}
            slots={slots.map((slot) => ({
              key: String(slot.start),
              state: slot.state,
              label: slot.state
                ? t("history.slot", { moment: moment(slot.start), state: root(`status.${slot.state}`) })
                : t("history.slotEmpty", { moment: moment(slot.start) }),
            }))}
          />
        </div>
        <TransitionsList transitions={data.transitions} now={now.getTime()} />
      </div>
    </SectionCard>
  );
}
