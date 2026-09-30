"use client";

import { GripVertical } from "lucide-react";
import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, useRef, useState } from "react";

import { DEFAULT_WIDTH, NARROWEST, clampWidth, widestFor } from "./panel-width";
import { setPanelWidth, usePanelWidth } from "./use-panel-width";

export const KEY_STEP = 16;
export const LONG_KEY_STEP = 64;

type Drag = { x: number; start: number; row: number };

function rowOf(handle: HTMLElement) {
  return handle.parentElement?.offsetWidth ?? 0;
}

export function PanelHandle() {
  const t = useTranslations("workflowEditor.panelHandle");
  const width = usePanelWidth();
  const [row, setRow] = useState<number | null>(null);
  const drag = useRef<Drag | null>(null);
  const shown = row === null ? width : clampWidth(width, row);

  const keyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const measured = rowOf(event.currentTarget);
    const base = clampWidth(width, measured);
    const step = event.shiftKey ? LONG_KEY_STEP : KEY_STEP;
    const moves: Record<string, number> = { ArrowLeft: base + step, ArrowRight: base - step, Home: NARROWEST, End: widestFor(measured) };
    if (moves[event.key] === undefined) {
      return;
    }
    event.preventDefault();
    setRow(measured);
    setPanelWidth(clampWidth(moves[event.key], measured), true);
  };

  const pointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const measured = rowOf(event.currentTarget);
    event.currentTarget.setPointerCapture?.(event.pointerId);
    setRow(measured);
    drag.current = { x: event.clientX, start: clampWidth(width, measured), row: measured };
  };

  const widthAt = (event: PointerEvent<HTMLDivElement>, from: Drag) => clampWidth(from.start + from.x - event.clientX, from.row);

  const pointerMove = (event: PointerEvent<HTMLDivElement>) => {
    if (drag.current !== null) {
      setPanelWidth(widthAt(event, drag.current), false);
    }
  };

  const pointerUp = (event: PointerEvent<HTMLDivElement>) => {
    if (drag.current !== null) {
      setPanelWidth(widthAt(event, drag.current), true);
      drag.current = null;
      event.currentTarget.releasePointerCapture?.(event.pointerId);
    }
  };

  return (
    <div
      role="separator"
      tabIndex={0}
      aria-orientation="vertical"
      aria-label={t("label")}
      aria-valuemin={NARROWEST}
      aria-valuemax={row === null ? undefined : widestFor(row)}
      aria-valuenow={shown}
      aria-valuetext={t("value", { width: shown })}
      title={t("hint")}
      data-panel-handle=""
      className="group/handle hidden w-3 shrink-0 cursor-col-resize touch-none items-center justify-center self-stretch rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring md:-mx-2 md:flex"
      onFocus={(event) => setRow(rowOf(event.currentTarget))}
      onKeyDown={keyDown}
      onPointerDown={pointerDown}
      onPointerMove={pointerMove}
      onPointerUp={pointerUp}
      onPointerCancel={pointerUp}
      onDoubleClick={() => setPanelWidth(DEFAULT_WIDTH, true)}
    >
      <GripVertical className="size-4 text-muted-foreground transition-colors group-hover/handle:text-primary group-focus-visible/handle:text-primary" aria-hidden />
    </div>
  );
}
