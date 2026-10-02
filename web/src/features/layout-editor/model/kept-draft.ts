import type { Dashboard, LayoutRequest } from "@/entities/dashboard";
import type { WidgetHeight } from "@/shared/api";

import { type Draft, fromLayout, toRequest, updateWidget } from "./draft";

export type PlaceSize = { width: number; height: WidgetHeight };

export const KEPT_DRAFT = "home-portal:layout-draft";

type Kept = { base: unknown; draft: Draft };

export function keepDraft(base: Draft, draft: Draft) {
  try {
    window.sessionStorage.setItem(KEPT_DRAFT, JSON.stringify({ base: toRequest(base), draft } satisfies Kept));
  } catch {
    return;
  }
}

export function dropDraft() {
  try {
    window.sessionStorage.removeItem(KEPT_DRAFT);
  } catch {
    return;
  }
}

export function keptDraft(base: Draft): Draft | null {
  try {
    const text = window.sessionStorage.getItem(KEPT_DRAFT);
    if (text === null) {
      return null;
    }
    const kept = JSON.parse(text) as Kept;
    return JSON.stringify(kept.base) === JSON.stringify(toRequest(base)) ? kept.draft : null;
  } catch {
    return null;
  }
}

export function isSavedPlace(place: string) {
  return place.startsWith("#");
}

export function resizedLayout(layout: Dashboard, place: string, size: PlaceSize): LayoutRequest {
  return toRequest(updateWidget(fromLayout(layout), place, size));
}

export function resizeKept(place: string, size: PlaceSize, saved: Dashboard | null) {
  try {
    const text = window.sessionStorage.getItem(KEPT_DRAFT);
    if (text === null) {
      return;
    }
    const kept = JSON.parse(text) as Kept;
    const base = saved === null ? kept.base : toRequest(fromLayout(saved));
    window.sessionStorage.setItem(KEPT_DRAFT, JSON.stringify({ base, draft: updateWidget(kept.draft, place, size) } satisfies Kept));
  } catch {
    return;
  }
}
