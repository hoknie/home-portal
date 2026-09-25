"use client";

import { ArrowRight } from "lucide-react";
import { useFormatter, useTranslations } from "next-intl";

import type { Transition } from "@/entities/service";
import { StatusBadge } from "@/shared/ui/status-badge";

import { durationParts } from "../model/history";

export type TransitionsListProps = { transitions: Transition[]; now: number };

export function TransitionsList({ transitions, now }: TransitionsListProps) {
  const t = useTranslations("servicePage");
  const format = useFormatter();
  const newest = [...transitions].reverse();
  return (
    <div className="grid gap-2">
      <h3 className="text-sm font-medium">{t("transitions.title")}</h3>
      {newest.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("transitions.empty")}</p>
      ) : (
        <ol className="grid gap-2">
          {newest.map((transition, index) => {
            const started = Date.parse(transition.at);
            const ended = index === 0 ? now : Date.parse(newest[index - 1].at);
            const duration = t("duration", durationParts(ended - started));
            return (
              <li key={transition.at} className="grid gap-1 rounded-lg border border-glass-edge bg-glass-tint p-3 text-sm">
                <div className="flex flex-wrap items-center gap-2">
                  <time dateTime={transition.at} className="tabular-nums text-muted-foreground">
                    {format.dateTime(new Date(transition.at), { dateStyle: "short", timeStyle: "short" })}
                  </time>
                  <StatusBadge state={transition.from} />
                  <ArrowRight className="size-3.5 text-muted-foreground" aria-hidden />
                  <StatusBadge state={transition.to} />
                  <span className="text-muted-foreground">
                    {index === 0 ? t("transitions.ongoing", { duration }) : t("transitions.lasted", { duration })}
                  </span>
                </div>
                {transition.error ? <p className="font-mono text-xs break-all text-muted-foreground">{transition.error}</p> : null}
              </li>
            );
          })}
        </ol>
      )}
    </div>
  );
}
