"use client";

import { useSyncExternalStore } from "react";

import { DEFAULT_WIDTH, readPanelWidth, writePanelWidth } from "./panel-width";

const listeners = new Set<() => void>();
let current: number | null = null;

function snapshot() {
  if (current === null) {
    current = readPanelWidth();
  }
  return current;
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function setPanelWidth(width: number, keep: boolean) {
  current = width;
  if (keep) {
    writePanelWidth(width);
  }
  for (const listener of listeners) {
    listener();
  }
}

export function rereadPanelWidth() {
  current = null;
  for (const listener of listeners) {
    listener();
  }
}

export function usePanelWidth() {
  return useSyncExternalStore(subscribe, snapshot, () => DEFAULT_WIDTH);
}
