"use client";

import { useSyncExternalStore } from "react";

import { type ModuleCategory, isCategory } from "@/entities/module";

export const CATEGORIES_KEY = "home-portal.menu-categories";

const NONE: ReadonlySet<ModuleCategory> = new Set();
const listeners = new Set<() => void>();
let current: ReadonlySet<ModuleCategory> | null = null;

export function readCollapsedCategories(): ReadonlySet<ModuleCategory> {
  if (typeof window === "undefined") {
    return NONE;
  }
  try {
    const stored: unknown = JSON.parse(window.localStorage.getItem(CATEGORIES_KEY) ?? "[]");
    return Array.isArray(stored) ? new Set(stored.filter(isCategory)) : NONE;
  } catch {
    return NONE;
  }
}

export function writeCollapsedCategories(collapsed: ReadonlySet<ModuleCategory>) {
  try {
    window.localStorage.setItem(CATEGORIES_KEY, JSON.stringify([...collapsed]));
  } catch {
    return;
  }
}

function snapshot() {
  if (current === null) {
    current = readCollapsedCategories();
  }
  return current;
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

function publish(next: ReadonlySet<ModuleCategory> | null) {
  current = next;
  for (const listener of listeners) {
    listener();
  }
}

export function toggleCategory(category: ModuleCategory) {
  const next = new Set(snapshot());
  if (next.has(category)) {
    next.delete(category);
  } else {
    next.add(category);
  }
  writeCollapsedCategories(next);
  publish(next);
}

export function rereadCollapsedCategories() {
  publish(null);
}

export function useCollapsedCategories() {
  return useSyncExternalStore(subscribe, snapshot, () => NONE);
}
