"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, useEffect, useRef } from "react";

import { type Target, cellAt, grabOf, sectionAt } from "../model/measure";
import { BareButton } from "@/shared/ui/kit";

export type Step = { columns: number; rows: number; sections: number };

export type MoveHandleProps = {
  title: string;
  section: string;
  tile: () => HTMLElement | null;
  onPreview: (target: Target | null) => void;
  onMove: (target: Target) => void;
  onStep: (step: Step) => void;
};

const KEYS: Record<string, Step> = {
  ArrowLeft: { columns: -1, rows: 0, sections: 0 },
  ArrowRight: { columns: 1, rows: 0, sections: 0 },
  ArrowUp: { columns: 0, rows: -1, sections: 0 },
  ArrowDown: { columns: 0, rows: 1, sections: 0 },
  PageUp: { columns: 0, rows: 0, sections: -1 },
  PageDown: { columns: 0, rows: 0, sections: 1 },
};

type Drag = { pointer: number; grab: { columns: number; rows: number }; target: Target | null };

export function MoveHandle({ title, section, tile, onPreview, onMove, onStep }: MoveHandleProps) {
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

  const down = (event: PointerEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.focus();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    const element = tile();
    drag.current = { pointer: event.pointerId, grab: element ? grabOf(element, section, event.clientX, event.clientY) : { columns: 0, rows: 0 }, target: null };
  };

  const move = (event: PointerEvent<HTMLButtonElement>) => {
    const current = drag.current;
    if (!current || current.pointer !== event.pointerId) {
      return;
    }
    const over = sectionAt(event.clientX, event.clientY) ?? section;
    const cell = cellAt(over, event.clientX, event.clientY, current.grab);
    if (!cell) {
      return;
    }
    const target = { section: over, ...cell };
    if (!current.target || current.target.section !== target.section || current.target.column !== target.column || current.target.row !== target.row) {
      drag.current = { ...current, target };
      onPreview(target);
    }
  };

  const up = (event: PointerEvent<HTMLButtonElement>) => {
    const current = drag.current;
    if (!current || current.pointer !== event.pointerId) {
      return;
    }
    drag.current = null;
    onPreview(null);
    if (current.target) {
      onMove(current.target);
    }
  };

  const key = (event: KeyboardEvent<HTMLButtonElement>) => {
    const stepped = KEYS[event.key];
    if (!stepped) {
      return;
    }
    event.preventDefault();
    onStep(stepped);
  };

  return (
    <BareButton
      aria-label={t("move", { title })}
      aria-keyshortcuts="ArrowLeft ArrowRight ArrowUp ArrowDown PageUp PageDown"
      data-move-handle=""
      className="group/move absolute inset-y-3 -left-2 z-10 flex w-4 cursor-grab touch-none items-center justify-center rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring max-sm:hidden"
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={up}
      onPointerCancel={cancel}
      onLostPointerCapture={() => drag.current && cancel()}
      onKeyDown={key}
    >
      <span className="h-8 w-1 rounded-full bg-border transition-colors group-hover/move:bg-primary group-focus-visible/move:bg-primary" />
    </BareButton>
  );
}
