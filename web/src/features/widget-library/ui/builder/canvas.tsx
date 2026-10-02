"use client";

import { useTranslations } from "next-intl";
import { type MouseEvent, type PointerEvent, type ReactNode, useState } from "react";

import type { WidgetHeight } from "@/shared/api";
import { Input } from "@/shared/ui/primitives";

import { CANVAS_ATTRIBUTE, CANVAS_ROOT, pathOf } from "../../model/block-drop";
import type { BlockPath } from "../../model/block-tree";

export type CanvasSize = { width: number; height: WidgetHeight };

export const ROW_PIXELS = 96;

function pathUnder(target: EventTarget | null): BlockPath | null {
  const element = target instanceof Element ? target.closest(`[${CANVAS_ATTRIBUTE}]`) : null;
  return element ? pathOf(element.getAttribute(CANVAS_ATTRIBUTE)) : null;
}

function NumberField({ label, value, min, max, empty, onValue }: { label: string; value: number | null; min: number; max: number; empty?: string; onValue: (value: number | null) => void }) {
  const [text, setText] = useState(value === null ? "" : String(value));
  const [shown, setShown] = useState(value);
  if (shown !== value) {
    setShown(value);
    setText(value === null ? "" : String(value));
  }
  return (
    <Input
      type="number"
      aria-label={label}
      title={label}
      min={min}
      max={max}
      className="h-8 w-14 px-2 text-center"
      placeholder={empty}
      value={text}
      onChange={(event) => {
        setText(event.target.value);
        const number = Number(event.target.value);
        if (event.target.value === "") {
          if (empty !== undefined) {
            onValue(null);
          }
        } else if (Number.isInteger(number) && number >= min && number <= max) {
          onValue(number);
        }
      }}
      onBlur={() => setText(value === null ? "" : String(value))}
    />
  );
}

export function SizeControls({ size, onSize }: { size: CanvasSize; onSize: (size: CanvasSize) => void }) {
  const t = useTranslations("widgetBuilder.canvas");
  return (
    <div className="flex items-center gap-2 text-xs text-muted-foreground" title={t("sizeSaved")}>
      <NumberField label={t("width")} value={size.width} min={1} max={12} onValue={(width) => onSize({ ...size, width: width ?? size.width })} />
      <span aria-hidden>{t("by")}</span>
      <NumberField label={t("height")} value={size.height === "auto" ? null : size.height} min={1} max={8} empty={t("auto")} onValue={(rows) => onSize({ ...size, height: rows ?? "auto" })} />
    </div>
  );
}

export type CanvasProps = {
  size: CanvasSize;
  empty: boolean;
  dragging: boolean;
  onSelect: (path: BlockPath | null) => void;
  onDragStart: (path: BlockPath, event: PointerEvent) => void;
  children: ReactNode;
};

export function Canvas({ size, empty, dragging, onSelect, onDragStart, children }: CanvasProps) {
  const t = useTranslations("widgetBuilder.canvas");
  const pick = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    onSelect(pathUnder(event.target));
  };
  return (
    <div className="grid gap-3" data-canvas="">
      <p className="text-xs text-muted-foreground">{t("hint")}</p>
      <div className="rounded-xl bg-[radial-gradient(circle,var(--color-border)_1px,transparent_1px)] bg-[length:16px_16px] p-6">
        <div
          {...{ [CANVAS_ROOT]: "" }}
          data-canvas-widget=""
          className={dragging ? "mx-auto min-w-0 rounded-2xl ring-2 ring-primary/30 ring-offset-4 ring-offset-transparent" : "mx-auto min-w-0"}
          style={{ width: `${(size.width / 12) * 100}%`, minWidth: "16rem", maxWidth: "100%", minHeight: "6rem", height: size.height === "auto" ? undefined : size.height * ROW_PIXELS - 16, overflow: size.height === "auto" ? undefined : "auto" }}
          onClickCapture={pick}
          onPointerDownCapture={(event) => {
            const path = pathUnder(event.target);
            if (path !== null) {
              onDragStart(path, event);
            }
          }}
        >
          {children}
          {empty ? <p className="py-8 text-center text-sm text-muted-foreground">{t("empty")}</p> : null}
        </div>
      </div>
    </div>
  );
}
