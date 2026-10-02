"use client";

import { useState } from "react";

function read(key: string, fallback: number) {
  try {
    const stored = Number(window.localStorage.getItem(key) ?? Number.NaN);
    return Number.isFinite(stored) && stored > 0 ? stored : fallback;
  } catch {
    return fallback;
  }
}

export function useStoredSize(key: string, fallback: number): [number, (value: number, done: boolean) => void] {
  const [size, setSize] = useState(() => (typeof window === "undefined" ? fallback : read(key, fallback)));
  const change = (value: number, done: boolean) => {
    setSize(value);
    if (done) {
      try {
        window.localStorage.setItem(key, String(Math.round(value)));
      } catch {
        return;
      }
    }
  };
  return [size, change];
}
