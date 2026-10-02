"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, useEffect, useRef } from "react";

import { GRID_COLUMNS, LARGEST_ROWS } from "@/shared/api";
import { cn } from "@/shared/lib/cn";

import { type Axis, type Size, sameSize, snapSize, steppedSize } from "../model/resize";

export type ResizeHandleProps = {
  axis: Axis;
  title: string;
  size: Size;
  measure: () => { gridWidth: number; startPixels: number };
  onPreview: (size: Size | null) => void;
  onResize: (size: Size) => void;
};

type Drag = { pointer: number; startX: number; startY: number; measured: { gridWidth: number; startPixels: number }; preview: Size };

const PLACES: Record<Axis, string> = {
  width: "inset-y-3 -right-2 w-4 cursor-ew-resize max-sm:hidden",
  height: "inset-x-6 -bottom-2 h-4 cursor-ns-resize",
  both: "-right-2 -bottom-2 size-5 cursor-nwse-resize max-sm:hidden",
};

const MARKS: Record<Axis, string> = {
  width: "h-8 w-1 rounded-full",
  height: "h-1 w-8 rounded-full",
  both: "size-2.5 rounded-sm",
};

export function useSizeText() {
  const t = useTranslations("layoutEditor");
  return {
    width: (size: Size) => t("columns", { count: size.width }),
    height: (size: Size) => (size.height === "auto" ? t("heightAuto") : t("rows", { count: size.height })),
    both: (size: Size) => t("sizeValue", { columns: size.width, rows: size.height === "auto" ? t("auto") : size.height }),
  };
}

export function ResizeHandle({ axis, title, size, measure, onPreview, onResize }: ResizeHandleProps) {
  const t = useTranslations("layoutEditor");
  const text = useSizeText();
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
    drag.current = { pointer: event.pointerId, startX: event.clientX, startY: event.clientY, measured: measure(), preview: size };
    onPreview(size);
  };

  const move = (event: PointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    if (!current || current.pointer !== event.pointerId) {
      return;
    }
    const preview = snapSize(size, axis, { x: event.clientX - current.startX, y: event.clientY - current.startY }, current.measured);
    if (!sameSize(preview, current.preview)) {
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
    if (!sameSize(current.preview, size)) {
      onResize(current.preview);
    }
  };

  const key = (event: KeyboardEvent<HTMLDivElement>) => {
    const next = steppedSize(size, axis, event.key);
    if (next === null) {
      return;
    }
    event.preventDefault();
    if (!sameSize(next, size)) {
      onResize(next);
    }
  };

  const vertical = axis === "height";
  return (
    <div
      role="slider"
      tabIndex={0}
      aria-label={t(axis === "width" ? "resize" : axis === "height" ? "resizeHeight" : "resizeBoth", { title })}
      aria-valuetext={text[axis](size)}
      aria-valuemin={vertical ? 0 : 1}
      aria-valuemax={vertical ? LARGEST_ROWS : GRID_COLUMNS}
      aria-valuenow={vertical ? (size.height === "auto" ? 0 : size.height) : size.width}
      aria-orientation={vertical ? "vertical" : "horizontal"}
      data-resize-handle={axis}
      className={cn(
        "group/resize absolute z-10 flex touch-none items-center justify-center rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring",
        PLACES[axis],
      )}
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={up}
      onPointerCancel={cancel}
      onLostPointerCapture={() => drag.current && cancel()}
      onKeyDown={key}
    >
      <span className={cn("bg-border transition-colors group-hover/resize:bg-primary group-focus-visible/resize:bg-primary", MARKS[axis])} />
    </div>
  );
}
