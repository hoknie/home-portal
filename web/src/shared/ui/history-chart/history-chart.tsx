"use client";

import { type KeyboardEvent, type PointerEvent, useId, useState } from "react";

import { cn } from "@/shared/lib/cn";
import { SURFACE } from "@/shared/ui/kit";

import { KNOWN_STATES, type KnownState, STATE_DOT } from "../status-badge";
import { AXIS_GUTTER, TimeAxis, valueTicks } from "../time-axis";
import { HEIGHT, type HistoryPoint, type Interval, WIDTH, bandsOf, intervalsOf, linesOf, markerOf, nearest, xOf } from "./plotting";

export type HistoryHint = { time: string; state: string; average: string; maximum: string };

export type HistoryChartProps = {
  points: HistoryPoint[];
  step: number;
  from: number;
  to: number;
  title: string;
  summary: string;
  empty: string;
  legend: { average: string; range: string; states: Record<KnownState, string> };
  formatTime: (at: number) => string;
  formatValue: (milliseconds: number) => string;
  hintOf: (interval: Interval) => HistoryHint;
};

const BAND_FILL: Record<KnownState, string> = {
  unknown: "fill-status-unknown",
  up: "fill-status-up",
  degraded: "fill-status-degraded",
  down: "fill-status-down",
  unreadable: "fill-status-unreadable",
};

const STEPS: Record<string, (index: number, last: number) => number> = {
  ArrowLeft: (index) => Math.max(0, index - 1),
  ArrowRight: (index, last) => Math.min(last, index + 1),
  Home: () => 0,
  End: (_, last) => last,
};

export function HistoryChart({ points, step, from, to, title, summary, empty, legend, formatTime, formatValue, hintOf }: HistoryChartProps) {
  const [chosen, setChosen] = useState<number | null>(null);
  const summaryId = useId();
  const fade = `fade-${summaryId.replace(/[^a-zA-Z0-9-]/g, "")}`;
  if (points.length === 0) {
    return <p className="py-8 text-center text-sm text-muted-foreground">{empty}</p>;
  }
  const intervals = intervalsOf(points, step, to);
  const highest = Math.max(0, ...intervals.map((interval) => Math.max(interval.maximum ?? 0, interval.average ?? 0)));
  const { ticks, top } = valueTicks(highest);
  const lines = linesOf(intervals, from, to, top);
  const bands = bandsOf(intervals);
  const states = KNOWN_STATES.filter((state) => bands.some((band) => band.state === state));
  const point = chosen === null ? null : (intervals[chosen] ?? null);
  const hint = point ? hintOf(point) : null;
  const position = point ? markerOf(point, from, to) / WIDTH : 0;
  const choose = (event: PointerEvent<HTMLDivElement>) => {
    const box = event.currentTarget.getBoundingClientRect();
    if (box.width > 0) {
      setChosen(nearest(intervals, from + ((event.clientX - box.left) / box.width) * (to - from)));
    }
  };
  const moveBy = (event: KeyboardEvent<HTMLDivElement>) => {
    const move = STEPS[event.key];
    if (move) {
      event.preventDefault();
      setChosen(move(chosen ?? intervals.length - 1, intervals.length - 1));
    }
  };
  return (
    <div className={AXIS_GUTTER}>
      <div aria-hidden className="relative my-2 h-32 text-right text-xs text-muted-foreground tabular-nums">
        {ticks.map((tick) => (
          <span key={tick} data-value-tick={tick} className="absolute right-0 -translate-y-1/2 whitespace-nowrap" style={{ top: `${((1 - tick / top) * 100).toFixed(3)}%` }}>
            {formatValue(tick)}
          </span>
        ))}
      </div>
      <div
        role="group"
        aria-label={title}
        aria-describedby={summaryId}
        tabIndex={0}
        className="relative my-2 rounded-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
        onPointerMove={choose}
        onPointerDown={choose}
        onPointerLeave={() => setChosen(null)}
        onKeyDown={moveBy}
        onBlur={() => setChosen(null)}
      >
        <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} preserveAspectRatio="none" aria-hidden className="h-32 w-full overflow-visible">
          {ticks.map((tick) => (
            <line key={tick} x1={0} x2={WIDTH} y1={HEIGHT - (tick / top) * HEIGHT} y2={HEIGHT - (tick / top) * HEIGHT} className={tick === 0 ? "stroke-border" : "stroke-border/50"} strokeWidth={1} vectorEffect="non-scaling-stroke" />
          ))}
          <defs>
            <linearGradient id={fade} x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stopColor="var(--color-primary)" stopOpacity={0.28} />
              <stop offset="100%" stopColor="var(--color-primary)" stopOpacity={0} />
            </linearGradient>
          </defs>
          {lines.map((current) => (
            <g key={current.line}>
              <path d={current.under} fill={`url(#${fade})`} data-under="" />
              <path d={current.area} className="fill-primary/10" data-range="" />
              <path d={current.line} fill="none" className="stroke-primary" strokeWidth={2.5} strokeLinejoin="round" strokeLinecap="round" vectorEffect="non-scaling-stroke" data-segment="" />
            </g>
          ))}
          {point ? <line x1={position * WIDTH} x2={position * WIDTH} y1={0} y2={HEIGHT} className="stroke-foreground/40" strokeWidth={1} vectorEffect="non-scaling-stroke" data-marker="" /> : null}
        </svg>
        <svg viewBox={`0 0 ${WIDTH} 10`} preserveAspectRatio="none" aria-hidden className="mt-1 h-2.5 w-full">
          {bands.map((band) => (
            <rect key={band.start} x={xOf(band.start, from, to)} y={0} width={Math.max(1, xOf(band.end, from, to) - xOf(band.start, from, to))} height={10} className={BAND_FILL[band.state]} data-band={band.state} />
          ))}
        </svg>
        {hint ? (
          <div
            className={cn(SURFACE.solid, "pointer-events-none absolute top-0 z-10 grid w-max max-w-56 -translate-x-1/2 gap-0.5 rounded-md px-2.5 py-1.5 text-xs")}
            style={{ left: `${Math.min(90, Math.max(10, position * 100)).toFixed(2)}%` }}
            data-hint=""
          >
            <span className="font-medium">{hint.time}</span>
            <span>{hint.state}</span>
            <span className="text-muted-foreground">{hint.average}</span>
            <span className="text-muted-foreground">{hint.maximum}</span>
          </div>
        ) : null}
      </div>
      <TimeAxis className="col-start-2" from={from} to={to} format={formatTime} />
      <ul className="col-start-2 mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground" aria-label={title}>
        <li className="flex items-center gap-1.5">
          <span aria-hidden className="h-[3px] w-4 rounded-full bg-primary" />
          {legend.average}
        </li>
        <li className="flex items-center gap-1.5">
          <span aria-hidden className="h-2.5 w-4 rounded-sm bg-primary/10" />
          {legend.range}
        </li>
        {states.map((state) => (
          <li key={state} className="flex items-center gap-1.5">
            <span aria-hidden className={cn("size-2.5 rounded-sm", STATE_DOT[state])} />
            {legend.states[state]}
          </li>
        ))}
      </ul>
      <p id={summaryId} className="sr-only">
        {summary}
      </p>
      <p aria-live="polite" className="sr-only">
        {hint ? `${hint.time}: ${hint.state}, ${hint.average}, ${hint.maximum}` : ""}
      </p>
    </div>
  );
}
