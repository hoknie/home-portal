"use client";

import { type RefObject, useLayoutEffect, useRef, useState } from "react";

import { rowsFor } from "./grid";

const remembered = new Map<string, number>();

export function rememberedRows(memory: string): number | undefined {
  return remembered.get(memory);
}

export function useAutoRows<T extends HTMLElement>(enabled: boolean, memory?: string): [RefObject<T | null>, number] {
  const ref = useRef<T>(null);
  const [rows, setRows] = useState(() => (memory === undefined ? undefined : remembered.get(memory)) ?? 1);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!enabled || !element) {
      return;
    }
    const measure = () => {
      const measured = rowsFor(element.getBoundingClientRect().height);
      if (memory !== undefined) {
        remembered.set(memory, measured);
      }
      setRows(measured);
    };
    measure();
    if (typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [enabled, memory]);
  return [ref, rows];
}
