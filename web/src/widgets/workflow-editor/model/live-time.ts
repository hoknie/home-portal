"use client";

import { useEffect, useState } from "react";

import type { TraceEntry } from "@/entities/automation";

export const TICK_MILLISECONDS = 1_000;

export function elapsedOf(entry: TraceEntry, receivedAt: number, now: number) {
  if (entry.outcome !== "running") {
    return entry.duration_milliseconds;
  }
  return entry.duration_milliseconds + Math.max(0, now - receivedAt);
}

export function timeLeftOf(entry: TraceEntry, receivedAt: number, now: number) {
  if (entry.wait_seconds === null) {
    return null;
  }
  return Math.max(0, entry.wait_seconds * TICK_MILLISECONDS - elapsedOf(entry, receivedAt, now));
}

export function useNow(ticking: boolean) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!ticking) {
      return;
    }
    const timer = window.setInterval(() => setNow(Date.now()), TICK_MILLISECONDS);
    return () => window.clearInterval(timer);
  }, [ticking]);
  return now;
}
