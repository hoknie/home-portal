"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, useEffect, useRef } from "react";

import { WIDGET_SIZES, type WidgetSize } from "@/shared/api";

import { snapSize, stepSize } from "../model/resize";

export type ResizeHandleProps = {
  title: string;
  size: WidgetSize;
  gridWidth: () => number;
  onPreview: (size: WidgetSize | null) => void;
  onResize: (size: WidgetSize) => void;
};

type Drag = { pointer: number; startX: number; width: number; preview: WidgetSize };

const STEPS: Record<string, number> = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1 };

export function ResizeHandle({ title, size, gridWidth, onPreview, onResize }: ResizeHandleProps) {
  const t = useTranslations("layoutEditor");
  const drag = useRef<Drag | null>(null);

  const cancel = () => {
    drag.current = null;
    onPreview(null);
  };

  useEffect(() => {
    const escape = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape" && drag.current) {
        cancel();
      }
    };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  });

  const down = (event: PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.focus();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    drag.current = { pointer: event.pointerId, startX: event.clientX, width: gridWidth(), preview: size };
    onPreview(size);
  };

  const move = (event: PointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    if (!current || current.pointer !== event.pointerId) {
      return;
    }
    const preview = snapSize(size, event.clientX - current.startX, current.width);
    if (preview !== current.preview) {
      drag.current = { ...current, preview };
      onPreview(preview);
    }
  };

  const up = (event: PointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    if (!current || current.pointer !== event.pointerId) {
      return;
    }
    drag.current = null;
    onPreview(null);
    if (current.preview !== size) {
      onResize(current.preview);
    }
  };

  const key = (event: KeyboardEvent<HTMLDivElement>) => {
    const next = event.key in STEPS ? stepSize(size, STEPS[event.key]) : event.key === "Home" ? WIDGET_SIZES[0] : event.key === "End" ? WIDGET_SIZES.at(-1) : null;
    if (next === null || next === undefined) {
      return;
    }
    event.preventDefault();
    if (next !== size) {
      onResize(next);
    }
  };

  return (
    <div
      role="slider"
      tabIndex={0}
      aria-label={t("resize", { title })}
      aria-orientation="horizontal"
      aria-valuemin={0}
      aria-valuemax={WIDGET_SIZES.length - 1}
      aria-valuenow={WIDGET_SIZES.indexOf(size)}
      aria-valuetext={t(`sizes.${size}`)}
      data-resize-handle=""
      className="group/resize absolute inset-y-3 -right-2 z-10 flex w-4 cursor-ew-resize touch-none items-center justify-center rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring max-sm:hidden"
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={up}
      onPointerCancel={cancel}
      onLostPointerCapture={() => drag.current && cancel()}
      onKeyDown={key}
    >
      <span className="h-8 w-1 rounded-full bg-border transition-colors group-hover/resize:bg-primary group-focus-visible/resize:bg-primary" />
    </div>
  );
}
