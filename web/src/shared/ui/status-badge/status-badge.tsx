"use client";

import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";

import { STATE_DOT, STATE_TONE, knownState } from "./states";

export type StatusProps = { state: string };

export function StatusDot({ state }: StatusProps) {
  const known = knownState(state);
  return (
    <span className="relative flex size-2.5 shrink-0" data-state={known}>
      {known === "up" ? <span className={cn("absolute inline-flex size-full animate-ping rounded-full opacity-60", STATE_DOT[known])} /> : null}
      <span className={cn("relative inline-flex size-2.5 rounded-full", STATE_DOT[known])} />
    </span>
  );
}

export function StatusBadge({ state }: StatusProps) {
  const t = useTranslations("status");
  const known = knownState(state);
  return (
    <span
      data-state={known}
      className={cn("inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-xs font-medium whitespace-nowrap", STATE_TONE[known])}
    >
      <StatusDot state={known} />
      {t(known)}
    </span>
  );
}
