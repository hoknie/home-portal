"use client";

import { useFormatter, useNow } from "next-intl";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

export type RelativeTimeProps = { moment: string | null; fallback?: string };

export function RelativeTime({ moment, fallback = "" }: RelativeTimeProps) {
  const format = useFormatter();
  const now = useNow({ updateInterval: STATUS_REFRESH_MILLISECONDS });
  if (!moment) {
    return <span>{fallback}</span>;
  }
  const parsed = new Date(moment);
  if (Number.isNaN(parsed.getTime())) {
    return <span>{fallback}</span>;
  }
  return <time dateTime={moment}>{format.relativeTime(parsed, now)}</time>;
}
