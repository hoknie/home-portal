"use client";

import { type RefObject, useEffect, useRef, useState } from "react";

import { rowsFor } from "./grid";

export function useAutoRows<T extends HTMLElement>(enabled: boolean): [RefObject<T | null>, number] {
  const ref = useRef<T>(null);
  const [rows, setRows] = useState(1);
  useEffect(() => {
    const element = ref.current;
    if (!enabled || !element || typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(() => setRows(rowsFor(element.getBoundingClientRect().height)));
    observer.observe(element);
    return () => observer.disconnect();
  }, [enabled]);
  return [ref, rows];
}
