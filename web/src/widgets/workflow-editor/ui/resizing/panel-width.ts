export const PANEL_KEY = "home-portal.workflow-panel-width";
export const DEFAULT_WIDTH = 416;
export const NARROWEST = 320;
export const WIDEST_SHARE = 0.7;

export function widestFor(row: number) {
  return Math.max(NARROWEST, Math.floor(row * WIDEST_SHARE));
}

export function clampWidth(width: number, row: number) {
  return Math.min(Math.max(Math.round(width), NARROWEST), widestFor(row));
}

export function readPanelWidth(): number {
  if (typeof window === "undefined") {
    return DEFAULT_WIDTH;
  }
  try {
    const stored = Number(window.localStorage.getItem(PANEL_KEY) ?? Number.NaN);
    return Number.isFinite(stored) && stored > 0 ? stored : DEFAULT_WIDTH;
  } catch {
    return DEFAULT_WIDTH;
  }
}

export function writePanelWidth(width: number) {
  try {
    window.localStorage.setItem(PANEL_KEY, String(Math.round(width)));
  } catch {
    return;
  }
}
