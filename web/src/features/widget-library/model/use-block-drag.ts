"use client";

import { type PointerEvent as ReactPointerEvent, useEffect, useRef, useState } from "react";

import type { RawBlock } from "./blocks";
import { type Drop, type DragSource, dropAt } from "./block-drop";
import { type Refusal, refusal } from "./block-tree";

export const DRAG_THRESHOLD = 4;

type Active = { source: DragSource; block: RawBlock; startX: number; startY: number; moving: boolean };

export type Ghost = { x: number; y: number; block: RawBlock };

export type BlockDrag = {
  start: (source: DragSource, block: RawBlock, event: ReactPointerEvent) => void;
  drop: Drop | null;
  refused: Refusal | null;
  dragging: boolean;
  ghost: Ghost | null;
  justDragged: () => boolean;
};

export function useBlockDrag(blocks: RawBlock[], onDrop: (source: DragSource, drop: Drop) => void, onRefused: (refused: Refusal) => void): BlockDrag {
  const active = useRef<Active | null>(null);
  const [drop, setDrop] = useState<Drop | null>(null);
  const [refused, setRefused] = useState<Refusal | null>(null);
  const [dragging, setDragging] = useState(false);
  const [ghost, setGhost] = useState<Ghost | null>(null);
  const dragged = useRef(false);
  const latest = useRef({ blocks, onDrop, onRefused, drop, refused });

  useEffect(() => {
    latest.current = { blocks, onDrop, onRefused, drop, refused };
  });

  useEffect(() => {
    const finish = () => {
      active.current = null;
      setDrop(null);
      setRefused(null);
      setDragging(false);
      setGhost(null);
    };
    const move = (event: PointerEvent) => {
      const current = active.current;
      if (!current) {
        return;
      }
      if (!current.moving && Math.hypot(event.clientX - current.startX, event.clientY - current.startY) < DRAG_THRESHOLD) {
        return;
      }
      current.moving = true;
      dragged.current = true;
      setDragging(true);
      setGhost({ x: event.clientX, y: event.clientY, block: current.block });
      const element = typeof document.elementFromPoint === "function" ? document.elementFromPoint(event.clientX, event.clientY) : null;
      const found = dropAt(latest.current.blocks, element, event.clientX, event.clientY);
      const from = current.source.kind === "move" ? current.source.path : null;
      const problem = found === null || found.replace ? null : refusal(latest.current.blocks, current.block, found.target, from);
      setRefused(problem);
      setDrop(problem === null ? found : null);
    };
    const up = () => {
      const current = active.current;
      const { drop: shown, refused: problem } = latest.current;
      if (current?.moving) {
        if (shown) {
          latest.current.onDrop(current.source, shown);
        } else if (problem) {
          latest.current.onRefused(problem);
        }
      }
      finish();
    };
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape" && active.current) {
        finish();
      }
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    window.addEventListener("keydown", key);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      window.removeEventListener("keydown", key);
    };
  }, []);

  return {
    start: (source, block, event) => {
      if (event.button !== 0) {
        return;
      }
      dragged.current = false;
      active.current = { source, block, startX: event.clientX, startY: event.clientY, moving: false };
    },
    drop,
    refused,
    dragging,
    ghost,
    justDragged: () => {
      const was = dragged.current;
      dragged.current = false;
      return was;
    },
  };
}
