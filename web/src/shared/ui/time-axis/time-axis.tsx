"use client";

import { useElementWidth } from "@/shared/lib/element-width";
import { cn } from "@/shared/lib/cn";

import { timeTicks } from "./ticks";

export type TimeAxisProps = { from: number; to: number; format: (at: number) => string; className?: string };

export function TimeAxis({ from, to, format, className }: TimeAxisProps) {
  const [ref, width] = useElementWidth<HTMLDivElement>();
  const ticks = timeTicks(from, to, width);
  return (
    <div ref={ref} aria-hidden className={cn("relative h-5 text-[11px] leading-5 text-muted-foreground tabular-nums", className)}>
      {ticks.map((tick) => (
        <span
          key={tick.at}
          data-tick={tick.at}
          className="absolute top-0 -translate-x-1/2 whitespace-nowrap before:absolute before:-top-1 before:left-1/2 before:h-1 before:border-l before:border-border"
          style={{ left: `${(tick.position * 100).toFixed(3)}%` }}
        >
          {format(tick.at)}
        </span>
      ))}
    </div>
  );
}
