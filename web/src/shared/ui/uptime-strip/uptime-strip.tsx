import { cn } from "@/shared/lib/cn";

import { STATE_DOT, type KnownState } from "../status-badge";
import { AXIS_GUTTER, TimeAxis } from "../time-axis";

export type UptimeSlot = { key: string; state: KnownState | null; label: string };

export type UptimeStripProps = { slots: UptimeSlot[]; title: string; from: number; to: number; formatTime: (at: number) => string };

export function UptimeStrip({ slots, title, from, to, formatTime }: UptimeStripProps) {
  return (
    <div className={AXIS_GUTTER}>
      <div role="img" aria-label={title} className="col-start-2 flex h-8 items-stretch gap-0.5">
        {slots.map((slot) => (
          <span
            key={slot.key}
            title={slot.label}
            data-state={slot.state ?? "none"}
            className={cn("min-w-0.5 flex-1 rounded-sm", slot.state ? STATE_DOT[slot.state] : "border border-dashed border-border bg-transparent")}
          />
        ))}
      </div>
      <TimeAxis className="col-start-2 mt-1" from={from} to={to} format={formatTime} />
    </div>
  );
}
