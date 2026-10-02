"use client";

import { type KeyboardEvent, type PointerEvent, useRef } from "react";

import { cn } from "@/shared/lib/cn";

export type SplitterProps = {
  label: string;
  orientation: "vertical" | "horizontal";
  value: number;
  min: number;
  max: number;
  direction?: 1 | -1;
  onChange: (value: number, done: boolean) => void;
  className?: string;
};

export const SPLIT_STEP = 16;

export function clampSize(value: number, min: number, max: number) {
  return Math.round(Math.min(Math.max(value, min), max));
}

export function Splitter({ label, orientation, value, min, max, direction = 1, onChange, className }: SplitterProps) {
  const start = useRef<{ at: number; value: number } | null>(null);
  const vertical = orientation === "vertical";
  const at = (event: PointerEvent) => (vertical ? event.clientX : event.clientY);
  const next = (event: PointerEvent) => {
    const from = start.current;
    return from === null ? value : clampSize(from.value + (at(event) - from.at) * direction, min, max);
  };
  const key = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = (event.shiftKey ? 4 : 1) * SPLIT_STEP;
    const less = vertical ? "ArrowLeft" : "ArrowUp";
    const more = vertical ? "ArrowRight" : "ArrowDown";
    const moves: Record<string, number> = { [less]: value - step * direction, [more]: value + step * direction, Home: min, End: max };
    if (moves[event.key] === undefined) {
      return;
    }
    event.preventDefault();
    onChange(clampSize(moves[event.key], min, max), true);
  };
  return (
    <div
      role="separator"
      tabIndex={0}
      aria-label={label}
      title={label}
      aria-orientation={orientation}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={Math.round(value)}
      data-splitter={orientation}
      className={cn(
        "group/split relative flex shrink-0 touch-none items-center justify-center rounded-full outline-none focus-visible:ring-2 focus-visible:ring-ring",
        vertical ? "w-2 cursor-col-resize self-stretch" : "h-2 cursor-row-resize",
        className,
      )}
      onKeyDown={key}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture?.(event.pointerId);
        start.current = { at: at(event), value };
      }}
      onPointerMove={(event) => {
        if (start.current !== null) {
          onChange(next(event), false);
        }
      }}
      onPointerUp={(event) => {
        if (start.current !== null) {
          onChange(next(event), true);
          start.current = null;
        }
      }}
      onPointerCancel={() => {
        start.current = null;
      }}
    >
      <span aria-hidden className={cn("rounded-full bg-border transition-colors group-hover/split:bg-primary group-focus-visible/split:bg-primary", vertical ? "h-10 w-1" : "h-1 w-10")} />
    </div>
  );
}
