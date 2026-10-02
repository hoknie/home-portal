"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, type PointerEvent, useEffect, useRef } from "react";

import { type Snapshot, grabbedColumns, placeAt, sectionAt, snapshotOf } from "../model/measure";
import type { Place, Step } from "../model/order";
import { BareButton } from "@/shared/ui/kit";

export type MoveHandleProps = {
  title: string;
  uid: string;
  section: string;
  sections: () => string[];
  tile: () => HTMLElement | null;
  onPreview: (place: Place | null) => void;
  onMove: (place: Place) => void;
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

type Drag = { pointer: number; grabbed: number; snapshot: Snapshot; place: Place | null };

const samePlace = (left: Place | null, right: Place) => left !== null && left.section === right.section && left.column === right.column && left.before === right.before;

export function MoveHandle({ title, uid, section, sections, tile, onPreview, onMove, onStep }: MoveHandleProps) {
  const t = useTranslations("layoutEditor");
  const drag = useRef<Drag | null>(null);
  const latest = useRef({ uid, section, onPreview, onMove });

  useEffect(() => {
    latest.current = { uid, section, onPreview, onMove };
  });

  useEffect(() => {
    const finish = () => {
      drag.current = null;
      latest.current.onPreview(null);
    };
    const move = (event: globalThis.PointerEvent) => {
      const current = drag.current;
      if (!current || current.pointer !== event.pointerId) {
        return;
      }
      const over = sectionAt(event.clientX, event.clientY) ?? latest.current.section;
      const place = placeAt(current.snapshot, over, latest.current.uid, event.clientX, event.clientY, current.grabbed);
      if (place && !samePlace(current.place, place)) {
        drag.current = { ...current, place };
        latest.current.onPreview(place);
      }
    };
    const up = (event: globalThis.PointerEvent) => {
      const current = drag.current;
      if (!current || current.pointer !== event.pointerId) {
        return;
      }
      finish();
      if (current.place) {
        latest.current.onMove(current.place);
      }
    };
    const escape = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape" && drag.current) {
        finish();
      }
    };
    const cancel = (event: globalThis.PointerEvent) => {
      if (drag.current?.pointer === event.pointerId) {
        finish();
      }
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    window.addEventListener("pointercancel", cancel);
    window.addEventListener("keydown", escape);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("pointercancel", cancel);
      window.removeEventListener("keydown", escape);
    };
  }, []);

  const down = (event: PointerEvent<HTMLButtonElement>) => {
    if (event.button !== 0) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.focus();
    const element = tile();
    drag.current = { pointer: event.pointerId, grabbed: element ? grabbedColumns(element, section, event.clientX) : 0, snapshot: snapshotOf(sections()), place: null };
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
      onKeyDown={key}
    >
      <span className="h-8 w-1 rounded-full bg-border transition-colors group-hover/move:bg-primary group-focus-visible/move:bg-primary" />
    </BareButton>
  );
}
