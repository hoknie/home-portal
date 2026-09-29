"use client";

import { ArrowRight } from "lucide-react";
import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";

import type { LogLevel, TraceEntry } from "../../model/schema";

const LEVEL_TONE: Record<LogLevel, string> = {
  info: "text-foreground",
  warning: "text-status-degraded",
  error: "text-status-down",
};

export function LevelMark({ level }: { level: LogLevel }) {
  const t = useTranslations("workflows.trace.levels");
  return <span className={cn("text-xs font-medium", LEVEL_TONE[level])}>{t(level)}</span>;
}

export function StepLog({ entry }: { entry: TraceEntry }) {
  const t = useTranslations("workflows.trace");
  return (
    <div className="grid gap-2 text-xs" data-testid="step-log">
      {entry.level ? (
        <p className={cn("font-mono break-all", LEVEL_TONE[entry.level])}>
          <LevelMark level={entry.level} /> {entry.detail}
        </p>
      ) : null}
      {entry.values.length > 0 ? (
        <section aria-label={t("values")} className="grid gap-1">
          <p className="text-muted-foreground">{t("values")}</p>
          <dl className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] gap-x-2 gap-y-1 font-mono">
            {entry.values.map((value, index) => (
              <div key={`${index}-${value.template}`} className="contents">
                <dt className="line-clamp-2 break-all text-muted-foreground" title={value.template}>
                  {value.template}
                </dt>
                <ArrowRight aria-hidden className="mt-0.5 size-3 text-muted-foreground" />
                <dd className="line-clamp-2 break-all" title={value.value}>
                  {value.value}
                </dd>
              </div>
            ))}
          </dl>
        </section>
      ) : null}
      {entry.values_dropped > 0 ? <p className="text-muted-foreground">{t("valuesDropped", { count: entry.values_dropped })}</p> : null}
      {entry.log.length > 0 ? (
        <section aria-label={t("log")} className="grid gap-1">
          <p className="text-muted-foreground">{t("log")}</p>
          <ol className="grid gap-0.5 font-mono">
            {entry.log.map((line, index) => (
              <li key={`${index}-${line}`} className="line-clamp-2 break-all" title={line}>
                {line}
              </li>
            ))}
          </ol>
        </section>
      ) : null}
      {entry.log_dropped > 0 ? <p className="text-muted-foreground">{t("logDropped", { count: entry.log_dropped })}</p> : null}
    </div>
  );
}
