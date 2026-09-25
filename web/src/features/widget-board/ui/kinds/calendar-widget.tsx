"use client";

import { CalendarDays, MapPin, Repeat } from "lucide-react";
import { useFormatter, useTranslations } from "next-intl";

import type { Calendar } from "@/entities/widget";
import { EmptyState } from "@/shared/ui/empty-state";

export type CalendarProps = { data: Calendar };

export function CalendarWidget({ data }: CalendarProps) {
  const t = useTranslations("widgets.calendar");
  const format = useFormatter();
  if (data.events.length === 0) {
    return <EmptyState icon={CalendarDays} title={t("empty")} description={t("emptyHint")} />;
  }
  return (
    <ul className="grid gap-2">
      {data.events.map((event, index) => (
        <li key={`${event.start}-${index}`} className="flex items-start gap-3 rounded-xl border bg-card p-3">
          <div className="grid min-w-16 text-center">
            <span className="text-xs text-muted-foreground">
              {format.dateTime(new Date(event.start), { weekday: "short" })}
            </span>
            <span className="text-lg font-semibold tabular-nums">
              {format.dateTime(new Date(event.start), { day: "numeric" })}
            </span>
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate font-medium">{event.summary}</p>
            <p className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
              <span>
                {event.all_day
                  ? t("allDay")
                  : format.dateTime(new Date(event.start), { hour: "2-digit", minute: "2-digit" })}
              </span>
              {event.location ? (
                <span className="inline-flex items-center gap-1">
                  <MapPin className="size-3" aria-hidden />
                  {event.location}
                </span>
              ) : null}
              {event.repeats === "unsupported" ? (
                <span className="inline-flex items-center gap-1" title={t("unsupportedRepeat")}>
                  <Repeat className="size-3" aria-hidden />
                  {t("repeats")}
                </span>
              ) : null}
            </p>
          </div>
        </li>
      ))}
    </ul>
  );
}
