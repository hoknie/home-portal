"use client";

import { useEffect, useState } from "react";

import { begin, isRedoKey, isUndoKey, nextRedo, nextUndo, pushed, redone, typingIn, undone } from "./history";

export type DraftHistory<Draft> = {
  draft: Draft;
  change: (label: string, update: (current: Draft) => Draft, coalesce?: boolean) => void;
  undo: () => void;
  redo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  reset: (draft: Draft) => void;
};

export function useDraftHistory<Draft>(
  base: Draft,
  same: (left: Draft, right: Draft) => boolean,
  announce: (kind: "undone" | "redone", label: string) => void,
): DraftHistory<Draft> {
  const [history, setHistory] = useState(() => begin(base));

  const change = (label: string, update: (current: Draft) => Draft, coalesce = false) =>
    setHistory((current) =>
      coalesce && nextUndo(current) === label && current.future.length === 0
        ? { ...current, present: update(current.present) }
        : pushed(current, update(current.present), label, same),
    );

  const undo = () => {
    const label = nextUndo(history);
    if (label !== null) {
      setHistory(undone);
      announce("undone", label);
    }
  };

  const redo = () => {
    const label = nextRedo(history);
    if (label !== null) {
      setHistory(redone);
      announce("redone", label);
    }
  };

  useEffect(() => {
    const pressed = (event: KeyboardEvent) => {
      if (typingIn(event.target)) {
        return;
      }
      if (isUndoKey(event)) {
        event.preventDefault();
        undo();
      } else if (isRedoKey(event)) {
        event.preventDefault();
        redo();
      }
    };
    window.addEventListener("keydown", pressed);
    return () => window.removeEventListener("keydown", pressed);
  });

  return {
    draft: history.present,
    change,
    undo,
    redo,
    canUndo: history.past.length > 0,
    canRedo: history.future.length > 0,
    reset: (draft) => setHistory(begin(draft)),
  };
}
