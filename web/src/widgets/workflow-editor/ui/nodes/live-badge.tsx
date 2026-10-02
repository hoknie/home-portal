"use client";

import { useTranslations } from "next-intl";

import type { TraceEntry } from "@/entities/automation";

import { TICK_MILLISECONDS, elapsedOf, timeLeftOf, useNow } from "../../model/live-time";

const RING_RADIUS = 7;
const RING_LENGTH = 2 * Math.PI * RING_RADIUS;

export function LiveBadge({ entry, receivedAt }: { entry: TraceEntry; receivedAt: number }) {
  const t = useTranslations("workflowEditor.run");
  const now = useNow(true);
  const left = timeLeftOf(entry, receivedAt, now);
  if (left !== null && entry.wait_seconds) {
    const seconds = Math.ceil(left / TICK_MILLISECONDS);
    const share = left / (entry.wait_seconds * TICK_MILLISECONDS);
    return (
      <span className="flex shrink-0 items-center gap-1 rounded-full bg-status-degraded/15 px-1.5 text-xs font-medium text-status-degraded tabular-nums" data-live="countdown" aria-label={t("secondsLeft", { seconds })}>
        <svg viewBox="0 0 18 18" className="size-3.5 -rotate-90" aria-hidden>
          <circle cx="9" cy="9" r={RING_RADIUS} fill="none" stroke="currentColor" strokeOpacity={0.25} strokeWidth={2.5} />
          <circle cx="9" cy="9" r={RING_RADIUS} fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeDasharray={RING_LENGTH} strokeDashoffset={RING_LENGTH * (1 - share)} className="transition-[stroke-dashoffset] duration-1000 ease-linear" />
        </svg>
        {t("seconds", { seconds })}
      </span>
    );
  }
  const seconds = Math.floor(elapsedOf(entry, receivedAt, now) / TICK_MILLISECONDS);
  return (
    <span className="flex shrink-0 items-center gap-1 rounded-full bg-status-degraded/15 px-1.5 text-xs font-medium text-status-degraded tabular-nums" data-live="elapsed" aria-label={t("runningFor", { seconds })}>
      <span className="size-1.5 animate-pulse rounded-full bg-status-degraded" aria-hidden />
      {t("seconds", { seconds })}
    </span>
  );
}
