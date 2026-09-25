import type { HistoryRange, LatencyPoint } from "@/entities/service";
import type { KnownState } from "@/shared/ui/status-badge";

export const SLOT_SECONDS: Record<HistoryRange, number> = { "24h": 3_600, "7d": 86_400, "30d": 86_400 };

export const SLOT_COUNT: Record<HistoryRange, number> = { "24h": 24, "7d": 7, "30d": 30 };

const SEVERITY: KnownState[] = ["unknown", "up", "degraded", "down", "unreadable"];

export type Slot = { start: number; state: KnownState | null };

export function slotsOf(points: LatencyPoint[], range: HistoryRange, now: number): Slot[] {
  const width = SLOT_SECONDS[range] * 1000;
  const count = SLOT_COUNT[range];
  const first = now - count * width;
  const slots: Slot[] = Array.from({ length: count }, (_, index) => ({ start: first + index * width, state: null }));
  for (const point of points) {
    const index = Math.floor((Date.parse(point.at) - first) / width);
    const slot = slots[index];
    if (!slot || point.state === "unknown") {
      continue;
    }
    if (slot.state === null || SEVERITY.indexOf(point.state) > SEVERITY.indexOf(slot.state)) {
      slot.state = point.state;
    }
  }
  return slots;
}

export function durationParts(milliseconds: number) {
  const minutes = Math.max(0, Math.round(milliseconds / 60_000));
  return { days: Math.floor(minutes / 1_440), hours: Math.floor((minutes % 1_440) / 60), minutes: minutes % 60 };
}

export function percent(ratio: number | null) {
  return ratio === null ? null : Math.floor(ratio * 1000) / 10;
}
